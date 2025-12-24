use super::super::{RdpClientCommand, RdpClientError, RdpClientEvent, RdpRect};
use super::connection::UpgradedFramed;
use ironrdp::cliprdr::CliprdrClient;
use ironrdp::connector::connection_activation::ConnectionActivationState;
use ironrdp::connector::ConnectionResult;
use ironrdp::graphics::image_processing::PixelFormat as IronPixelFormat;
use ironrdp::pdu::input::fast_path::{FastPathInputEvent, KeyboardFlags};
use ironrdp::pdu::input::mouse::PointerFlags;
use ironrdp::pdu::input::MousePdu;
use ironrdp::pdu::WriteBuf;
use ironrdp::session::image::DecodedImage;
use ironrdp::session::{fast_path, ActiveStage, ActiveStageOutput};
use ironrdp_tokio::{single_sequence_step_read, split_tokio_framed, FramedWrite};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Runs the active RDP session, processing framebuffer updates and input
// The future is not Send because IronRDP's AsyncNetworkClient is not Send.
// This is fine because we run on a single-threaded Tokio runtime.
#[allow(clippy::future_not_send)]
#[allow(clippy::too_many_lines)]
pub async fn run_active_session(
    framed: UpgradedFramed,
    connection_result: ConnectionResult,
    event_tx: std::sync::mpsc::Sender<RdpClientEvent>,
    command_rx: std::sync::mpsc::Receiver<RdpClientCommand>,
    shutdown_signal: Arc<AtomicBool>,
) -> Result<(), RdpClientError> {
    let (mut reader, mut writer) = split_tokio_framed(framed);

    // Create decoded image buffer
    let mut image = DecodedImage::new(
        IronPixelFormat::BgrA32,
        connection_result.desktop_size.width,
        connection_result.desktop_size.height,
    );

    let mut active_stage = ActiveStage::new(connection_result);

    loop {
        // Check shutdown signal
        if shutdown_signal.load(Ordering::SeqCst) {
            if let Ok(frames) = active_stage.graceful_shutdown() {
                for frame in frames {
                    if let ActiveStageOutput::ResponseFrame(data) = frame {
                        let _ = writer.write_all(&data).await;
                    }
                }
            }
            break;
        }

        // Process commands from GUI (non-blocking)
        while let Ok(cmd) = command_rx.try_recv() {
            if process_command(cmd, &mut active_stage, &mut image, &mut writer).await? {
                return Ok(());
            }
        }

        // Read and process RDP frames with timeout
        let read_result = tokio::time::timeout(
            std::time::Duration::from_millis(16), // ~60 FPS
            reader.read_pdu(),
        )
        .await;

        match read_result {
            Ok(Ok((action, payload))) => {
                match active_stage.process(&mut image, action, &payload) {
                    Ok(outputs) => {
                        for output in outputs {
                            match output {
                                ActiveStageOutput::ResponseFrame(data) => {
                                    if let Err(e) = writer.write_all(&data).await {
                                        return Err(RdpClientError::ConnectionFailed(format!(
                                            "Write error: {e}"
                                        )));
                                    }
                                }
                                ActiveStageOutput::GraphicsUpdate(region) => {
                                    let rect = RdpRect::new(
                                        region.left,
                                        region.top,
                                        region.right.saturating_sub(region.left),
                                        region.bottom.saturating_sub(region.top),
                                    );
                                    let data = extract_region_data(&image, rect);
                                    let _ =
                                        event_tx.send(RdpClientEvent::FrameUpdate { rect, data });
                                }
                                ActiveStageOutput::PointerDefault => {
                                    let _ = event_tx.send(RdpClientEvent::CursorDefault);
                                }
                                ActiveStageOutput::PointerHidden => {
                                    let _ = event_tx.send(RdpClientEvent::CursorHidden);
                                }
                                ActiveStageOutput::PointerPosition { x, y } => {
                                    let _ = event_tx.send(RdpClientEvent::CursorPosition { x, y });
                                }
                                ActiveStageOutput::PointerBitmap(pointer) => {
                                    let _ = event_tx.send(RdpClientEvent::CursorUpdate {
                                        width: pointer.width,
                                        height: pointer.height,
                                        hotspot_x: pointer.hotspot_x,
                                        hotspot_y: pointer.hotspot_y,
                                        data: pointer.bitmap_data.clone(),
                                    });
                                }
                                ActiveStageOutput::Terminate(reason) => {
                                    tracing::info!("RDP session terminated: {reason:?}");
                                    return Ok(());
                                }
                                ActiveStageOutput::DeactivateAll(mut connection_activation) => {
                                    // Execute the Deactivation-Reactivation Sequence:
                                    // https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/dfc234ce-481a-4674-9a5d-2a7bafb14432
                                    tracing::debug!(
                                        "Received Server Deactivate All PDU, \
                                         executing Deactivation-Reactivation Sequence"
                                    );

                                    let mut buf = WriteBuf::new();
                                    loop {
                                        let written = match single_sequence_step_read(
                                            &mut reader,
                                            &mut *connection_activation,
                                            &mut buf,
                                        )
                                        .await
                                        {
                                            Ok(w) => w,
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Reactivation sequence error: {}",
                                                    e
                                                );
                                                break;
                                            }
                                        };

                                        if written.size().is_some() {
                                            if let Err(e) = writer.write_all(buf.filled()).await {
                                                tracing::warn!(
                                                    "Failed to send reactivation response: {}",
                                                    e
                                                );
                                                break;
                                            }
                                        }

                                        if let ConnectionActivationState::Finalized {
                                            io_channel_id,
                                            user_channel_id,
                                            desktop_size,
                                            enable_server_pointer,
                                            pointer_software_rendering,
                                        } = connection_activation.state
                                        {
                                            tracing::debug!(
                                                ?desktop_size,
                                                "Deactivation-Reactivation Sequence completed"
                                            );

                                            // Update image size with the new desktop size
                                            image = DecodedImage::new(
                                                IronPixelFormat::BgrA32,
                                                desktop_size.width,
                                                desktop_size.height,
                                            );

                                            // Update the active stage with new channel IDs
                                            // and pointer settings
                                            active_stage.set_fastpath_processor(
                                                fast_path::ProcessorBuilder {
                                                    io_channel_id,
                                                    user_channel_id,
                                                    enable_server_pointer,
                                                    pointer_software_rendering,
                                                }
                                                .build(),
                                            );
                                            active_stage.set_enable_server_pointer(enable_server_pointer);

                                            // Notify GUI about resolution change
                                            let _ =
                                                event_tx.send(RdpClientEvent::ResolutionChanged {
                                                    width: desktop_size.width,
                                                    height: desktop_size.height,
                                                });

                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(RdpClientError::ProtocolError(format!("Session error: {e}")));
                    }
                }
            }
            Ok(Err(e)) => {
                return Err(RdpClientError::ConnectionFailed(format!("Read error: {e}")));
            }
            Err(_) => {
                // Timeout - no data available, continue loop
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn process_command<W: FramedWrite>(
    cmd: RdpClientCommand,
    active_stage: &mut ActiveStage,
    image: &mut DecodedImage,
    writer: &mut W,
) -> Result<bool, RdpClientError> {
    match cmd {
        RdpClientCommand::Disconnect => {
            if let Ok(frames) = active_stage.graceful_shutdown() {
                for frame in frames {
                    if let ActiveStageOutput::ResponseFrame(data) = frame {
                        let _ = writer.write_all(&data).await;
                    }
                }
            }
            return Ok(true);
        }
        RdpClientCommand::KeyEvent {
            scancode,
            pressed,
            extended,
        } => {
            let event = create_keyboard_event(scancode, pressed, extended);
            send_input_events(active_stage, image, writer, &[event]).await;
        }
        RdpClientCommand::UnicodeEvent { character, pressed } => {
            let event = create_unicode_event(character, pressed);
            send_input_events(active_stage, image, writer, &[event]).await;
        }
        RdpClientCommand::PointerEvent { x, y, buttons } => {
            let event = create_pointer_event(x, y, buttons);
            send_input_events(active_stage, image, writer, &[event]).await;
        }
        RdpClientCommand::MouseButtonPress { x, y, button } => {
            let event = create_button_press_event(x, y, button);
            send_input_events(active_stage, image, writer, &[event]).await;
        }
        RdpClientCommand::MouseButtonRelease { x, y, button } => {
            let event = create_button_release_event(x, y, button);
            send_input_events(active_stage, image, writer, &[event]).await;
        }
        RdpClientCommand::SendCtrlAltDel => {
            let events = create_ctrl_alt_del_sequence();
            send_input_events(active_stage, image, writer, &events).await;
        }
        RdpClientCommand::WheelEvent {
            horizontal,
            vertical,
        } => {
            if vertical != 0 {
                let event = create_wheel_event(vertical, false);
                send_input_events(active_stage, image, writer, &[event]).await;
            }
            if horizontal != 0 {
                let event = create_wheel_event(horizontal, true);
                send_input_events(active_stage, image, writer, &[event]).await;
            }
        }
        RdpClientCommand::SetDesktopSize { width, height } => {
            if let Some(result) =
                active_stage.encode_resize(u32::from(width), u32::from(height), None, None)
            {
                match result {
                    Ok(frame) => {
                        let _ = writer.write_all(&frame).await;
                        tracing::debug!("Resolution change requested: {}x{}", width, height);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to encode resize request: {}", e);
                    }
                }
            } else {
                tracing::debug!("Display Control not available for resize {}x{}", width, height);
            }
        }
        RdpClientCommand::RefreshScreen => {
            tracing::debug!("Screen refresh requested");
        }
        RdpClientCommand::ClipboardText(text) => {
            tracing::debug!("Pasting {} chars via Unicode key events", text.len());
            for ch in text.chars() {
                let event_press = create_unicode_event(ch, true);
                let event_release = create_unicode_event(ch, false);
                send_input_events(active_stage, image, writer, &[event_press]).await;
                send_input_events(active_stage, image, writer, &[event_release]).await;
            }
        }
        RdpClientCommand::Authenticate { .. } => {}
        RdpClientCommand::ClipboardData { format_id, data } => {
            if let Some(cliprdr) = active_stage.get_svc_processor_mut::<CliprdrClient>() {
                let response =
                    ironrdp::cliprdr::pdu::OwnedFormatDataResponse::new_data(data.clone());
                if let Ok(messages) = cliprdr.submit_format_data(response) {
                    if let Ok(frame) = active_stage.process_svc_processor_messages(messages) {
                        let _ = writer.write_all(&frame).await;
                        tracing::debug!(
                            "Clipboard data sent for format {}: {} bytes",
                            format_id,
                            data.len()
                        );
                    }
                }
            }
        }
        RdpClientCommand::ClipboardCopy(formats) => {
            if let Some(cliprdr) = active_stage.get_svc_processor_mut::<CliprdrClient>() {
                let clipboard_formats: Vec<ironrdp::cliprdr::pdu::ClipboardFormat> = formats
                    .iter()
                    .map(|f| {
                        let mut format = ironrdp::cliprdr::pdu::ClipboardFormat::new(
                            ironrdp::cliprdr::pdu::ClipboardFormatId::new(f.id),
                        );
                        if let Some(ref name) = f.name {
                            format = format.with_name(
                                ironrdp::cliprdr::pdu::ClipboardFormatName::new(name.clone()),
                            );
                        }
                        format
                    })
                    .collect();
                if let Ok(messages) = cliprdr.initiate_copy(&clipboard_formats) {
                    if let Ok(frame) = active_stage.process_svc_processor_messages(messages) {
                        let _ = writer.write_all(&frame).await;
                        tracing::debug!("Clipboard copy initiated with {} formats", formats.len());
                    }
                }
            }
        }
        RdpClientCommand::RequestClipboardData { format_id } => {
            tracing::debug!("RequestClipboardData command received for format {}", format_id);
            if let Some(cliprdr) = active_stage.get_svc_processor_mut::<CliprdrClient>() {
                let format = ironrdp::cliprdr::pdu::ClipboardFormatId::new(format_id);
                match cliprdr.initiate_paste(format) {
                    Ok(messages) => {
                        tracing::debug!("initiate_paste succeeded");
                        if let Ok(frame) = active_stage.process_svc_processor_messages(messages) {
                            let _ = writer.write_all(&frame).await;
                            tracing::debug!("Clipboard paste request sent for format {}", format_id);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("initiate_paste failed: {}", e);
                    }
                }
            } else {
                tracing::warn!("CLIPRDR channel not available");
            }
        }
    }
    Ok(false)
}

/// Creates a keyboard `FastPath` event
fn create_keyboard_event(scancode: u16, pressed: bool, extended: bool) -> FastPathInputEvent {
    let mut flags = KeyboardFlags::empty();
    if !pressed {
        flags |= KeyboardFlags::RELEASE;
    }
    if extended {
        flags |= KeyboardFlags::EXTENDED;
    }
    // RDP scancodes are 8-bit, but we use u16 to preserve the value during transmission
    // The actual scancode is in the lower 8 bits
    FastPathInputEvent::KeyboardEvent(flags, scancode as u8)
}

/// Creates a Unicode keyboard `FastPath` event for non-ASCII characters
fn create_unicode_event(character: char, pressed: bool) -> FastPathInputEvent {
    let mut flags = KeyboardFlags::empty();
    if !pressed {
        flags |= KeyboardFlags::RELEASE;
    }
    // Unicode events use the character's code point as u16
    // Characters outside BMP (> 0xFFFF) are truncated, but most keyboard input is within BMP
    let code_point = character as u32 as u16;
    FastPathInputEvent::UnicodeKeyboardEvent(flags, code_point)
}

/// Creates a pointer/mouse motion `FastPath` event (no button state change)
const fn create_pointer_event(x: u16, y: u16, _buttons: u8) -> FastPathInputEvent {
    // For motion events, only send MOVE flag - no button state
    FastPathInputEvent::MouseEvent(MousePdu {
        flags: PointerFlags::MOVE,
        number_of_wheel_rotation_units: 0,
        x_position: x,
        y_position: y,
    })
}

/// Creates a mouse button press `FastPath` event
fn create_button_press_event(x: u16, y: u16, button: u8) -> FastPathInputEvent {
    let button_flag = match button {
        2 => PointerFlags::RIGHT_BUTTON,
        3 => PointerFlags::MIDDLE_BUTTON_OR_WHEEL,
        _ => PointerFlags::LEFT_BUTTON,
    };

    // Button press: button flag + DOWN, no MOVE
    FastPathInputEvent::MouseEvent(MousePdu {
        flags: button_flag | PointerFlags::DOWN,
        number_of_wheel_rotation_units: 0,
        x_position: x,
        y_position: y,
    })
}

/// Creates a mouse button release `FastPath` event
const fn create_button_release_event(x: u16, y: u16, button: u8) -> FastPathInputEvent {
    let button_flag = match button {
        2 => PointerFlags::RIGHT_BUTTON,
        3 => PointerFlags::MIDDLE_BUTTON_OR_WHEEL,
        _ => PointerFlags::LEFT_BUTTON,
    };

    // Button release: only button flag, no DOWN, no MOVE
    FastPathInputEvent::MouseEvent(MousePdu {
        flags: button_flag,
        number_of_wheel_rotation_units: 0,
        x_position: x,
        y_position: y,
    })
}

/// Creates Ctrl+Alt+Del key sequence
fn create_ctrl_alt_del_sequence() -> [FastPathInputEvent; 6] {
    [
        // Ctrl down
        FastPathInputEvent::KeyboardEvent(KeyboardFlags::empty(), 0x1D),
        // Alt down
        FastPathInputEvent::KeyboardEvent(KeyboardFlags::empty(), 0x38),
        // Delete down (extended)
        FastPathInputEvent::KeyboardEvent(KeyboardFlags::EXTENDED, 0x53),
        // Delete up
        FastPathInputEvent::KeyboardEvent(KeyboardFlags::RELEASE | KeyboardFlags::EXTENDED, 0x53),
        // Alt up
        FastPathInputEvent::KeyboardEvent(KeyboardFlags::RELEASE, 0x38),
        // Ctrl up
        FastPathInputEvent::KeyboardEvent(KeyboardFlags::RELEASE, 0x1D),
    ]
}

/// Creates a mouse wheel event
const fn create_wheel_event(delta: i16, horizontal: bool) -> FastPathInputEvent {
    let flags = if horizontal {
        PointerFlags::HORIZONTAL_WHEEL
    } else {
        PointerFlags::VERTICAL_WHEEL
    };

    FastPathInputEvent::MouseEvent(MousePdu {
        flags,
        number_of_wheel_rotation_units: delta,
        x_position: 0,
        y_position: 0,
    })
}

/// Sends input events to the RDP server
async fn send_input_events<W: FramedWrite>(
    active_stage: &mut ActiveStage,
    image: &mut DecodedImage,
    writer: &mut W,
    events: &[FastPathInputEvent],
) {
    if let Ok(outputs) = active_stage.process_fastpath_input(image, events) {
        for output in outputs {
            if let ActiveStageOutput::ResponseFrame(data) = output {
                let _ = writer.write_all(&data).await;
            }
        }
    }
}

/// Extracts pixel data for a specific region from the decoded image
/// Converts from `IronRDP`'s BGRA format to Cairo's ARGB32 format by swapping R and B channels
fn extract_region_data(image: &DecodedImage, rect: RdpRect) -> Vec<u8> {
    let img_width = image.width();
    let img_height = image.height();
    let data = image.data();

    let region_x = rect.x.min(img_width);
    let region_y = rect.y.min(img_height);
    let region_w = rect.width.min(img_width.saturating_sub(region_x));
    let region_h = rect.height.min(img_height.saturating_sub(region_y));

    if region_w == 0 || region_h == 0 {
        return Vec::new();
    }

    let bytes_per_pixel = 4;
    let stride = img_width as usize * bytes_per_pixel;
    let result_size = region_w as usize * region_h as usize * bytes_per_pixel;
    let mut result = vec![0u8; result_size];

    for row in 0..region_h as usize {
        let src_row = (region_y as usize + row) * stride + region_x as usize * bytes_per_pixel;
        let dst_row = row * region_w as usize * bytes_per_pixel;

        for col in 0..region_w as usize {
            let src_idx = src_row + col * bytes_per_pixel;
            let dst_idx = dst_row + col * bytes_per_pixel;

            if src_idx + 4 <= data.len() {
                // IronRDP outputs BGRA, Cairo expects ARGB32 (which is BGRA on little-endian)
                // However, we observe swapped colors (Blue <-> Red), so we need to swap them manually.
                // This suggests either IronRDP or the server is producing RGBA order.
                result[dst_idx] = data[src_idx + 2];     // B
                result[dst_idx + 1] = data[src_idx + 1]; // G
                result[dst_idx + 2] = data[src_idx];     // R
                result[dst_idx + 3] = data[src_idx + 3]; // A
            }
        }
    }

    result
}
