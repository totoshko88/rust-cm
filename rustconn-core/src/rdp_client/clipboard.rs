//! RDP Clipboard backend implementation
//!
//! This module implements the `CliprdrBackend` trait from `IronRDP`
//! to handle clipboard operations between client and server.

use super::{ClipboardFormatInfo, RdpClientEvent};
use ironrdp::cliprdr::backend::{ClipboardMessage, ClipboardMessageProxy, CliprdrBackend};
use ironrdp::cliprdr::pdu::{
    ClipboardFormat, ClipboardFormatId, ClipboardGeneralCapabilityFlags, FileContentsRequest,
    FileContentsResponse, FormatDataRequest, FormatDataResponse, LockDataId,
};
use ironrdp::core::impl_as_any;
use std::sync::mpsc::Sender;
use tracing::{debug, trace, warn};

/// Proxy for sending clipboard messages to the main event loop
#[derive(Clone, Debug)]
pub struct RustConnClipboardProxy {
    event_tx: Sender<RdpClientEvent>,
}

impl RustConnClipboardProxy {
    /// Creates a new clipboard proxy
    #[must_use]
    pub const fn new(event_tx: Sender<RdpClientEvent>) -> Self {
        Self { event_tx }
    }
}

impl ClipboardMessageProxy for RustConnClipboardProxy {
    fn send_clipboard_message(&self, message: ClipboardMessage) {
        match message {
            ClipboardMessage::SendInitiateCopy(formats) => {
                // Backend wants to send format list to server (initiate copy)
                let format_infos: Vec<ClipboardFormatInfo> = formats
                    .iter()
                    .map(|f| {
                        let name = f.name.as_ref().map(|n| format!("{n:?}"));
                        ClipboardFormatInfo::new(f.id.value(), name)
                    })
                    .collect();
                trace!("Sending ClipboardCopy with {} formats", format_infos.len());
                let _ = self
                    .event_tx
                    .send(RdpClientEvent::ClipboardInitiateCopy(format_infos));
            }
            ClipboardMessage::SendInitiatePaste(format_id) => {
                trace!("Requesting clipboard data for format {}", format_id.value());
                let format_info = ClipboardFormatInfo::new(format_id.value(), None);
                let _ = self
                    .event_tx
                    .send(RdpClientEvent::ClipboardPasteRequest(format_info));
            }
            ClipboardMessage::SendFormatData(response) => {
                // This is called when IronRDP wants us to send data to server
                // But we also use it to extract received data
                let data = response.data();
                trace!(
                    "SendFormatData called with {} bytes (this is for sending TO server)",
                    data.len()
                );
            }
            ClipboardMessage::Error(err) => {
                warn!("Clipboard error: {}", err);
            }
        }
    }
}

/// `RustConn` clipboard backend for `IronRDP`
#[derive(Debug)]
pub struct RustConnClipboardBackend {
    proxy: RustConnClipboardProxy,
    ready: bool,
    pending_paste_format: Option<ClipboardFormatId>,
}

impl_as_any!(RustConnClipboardBackend);

impl RustConnClipboardBackend {
    /// Creates a new clipboard backend
    #[must_use]
    pub const fn new(event_tx: Sender<RdpClientEvent>) -> Self {
        Self {
            proxy: RustConnClipboardProxy::new(event_tx),
            ready: false,
            pending_paste_format: None,
        }
    }

    /// Returns true if the clipboard is ready
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.ready
    }
}

impl CliprdrBackend for RustConnClipboardBackend {
    #[allow(clippy::unnecessary_literal_bound)]
    fn temporary_directory(&self) -> &str {
        ".cliprdr"
    }

    fn on_ready(&mut self) {
        debug!("Clipboard channel ready");
        self.ready = true;
    }

    fn on_request_format_list(&mut self) {
        trace!("Server requested format list - sending empty list to complete initialization");
        // Send an empty format list to complete the initialization handshake
        self.proxy
            .send_clipboard_message(ClipboardMessage::SendInitiateCopy(Vec::new()));
    }

    fn client_capabilities(&self) -> ClipboardGeneralCapabilityFlags {
        ClipboardGeneralCapabilityFlags::empty()
    }

    fn on_process_negotiated_capabilities(
        &mut self,
        capabilities: ClipboardGeneralCapabilityFlags,
    ) {
        trace!(?capabilities, "Negotiated clipboard capabilities");
    }

    fn on_remote_copy(&mut self, available_formats: &[ClipboardFormat]) {
        debug!(
            "on_remote_copy called with {} formats: {:?}",
            available_formats.len(),
            available_formats
                .iter()
                .map(|f| f.id.value())
                .collect::<Vec<_>>()
        );

        // Notify GUI about available formats (for UI display)
        let format_infos: Vec<ClipboardFormatInfo> = available_formats
            .iter()
            .map(|f| {
                let name = f.name.as_ref().map(|n| format!("{n:?}"));
                ClipboardFormatInfo::new(f.id.value(), name)
            })
            .collect();
        let _ = self
            .proxy
            .event_tx
            .send(RdpClientEvent::ClipboardFormatsAvailable(format_infos));

        // Check if text format is available and auto-request it
        let text_format = available_formats
            .iter()
            .find(|f| f.id == ClipboardFormatId::CF_UNICODETEXT)
            .or_else(|| {
                available_formats
                    .iter()
                    .find(|f| f.id == ClipboardFormatId::CF_TEXT)
            });

        if let Some(format) = text_format {
            debug!(
                "Text format available (id={}), requesting paste",
                format.id.value()
            );
            self.pending_paste_format = Some(format.id);
            self.proxy
                .send_clipboard_message(ClipboardMessage::SendInitiatePaste(format.id));
        } else {
            debug!("No text format available in clipboard");
        }
    }

    fn on_format_data_request(&mut self, request: FormatDataRequest) {
        trace!(?request, "Format data request from server");
        self.proxy
            .send_clipboard_message(ClipboardMessage::SendInitiatePaste(request.format));
    }

    fn on_format_data_response(&mut self, response: FormatDataResponse<'_>) {
        let data = response.data();
        let format_id = self.pending_paste_format.take();
        debug!(
            "on_format_data_response called: {} bytes, format: {:?}",
            data.len(),
            format_id
        );

        match format_id {
            Some(ClipboardFormatId::CF_UNICODETEXT) | None => {
                if let Ok(text) = string_from_utf16(data) {
                    debug!("Clipboard text decoded (UTF-16): {} chars", text.len());
                    let _ = self
                        .proxy
                        .event_tx
                        .send(RdpClientEvent::ClipboardText(text));
                } else {
                    warn!("Failed to decode clipboard data as UTF-16");
                }
            }
            Some(ClipboardFormatId::CF_TEXT) => {
                if let Ok(text) = String::from_utf8(data.to_vec()) {
                    debug!("Clipboard text decoded (ANSI): {} chars", text.len());
                    let _ = self
                        .proxy
                        .event_tx
                        .send(RdpClientEvent::ClipboardText(text));
                } else {
                    let text: String = data.iter().map(|&b| b as char).collect();
                    debug!("Clipboard text decoded (Latin-1): {} chars", text.len());
                    let _ = self
                        .proxy
                        .event_tx
                        .send(RdpClientEvent::ClipboardText(text));
                }
            }
            Some(_) => {
                if let Ok(text) = string_from_utf16(data) {
                    debug!("Clipboard text decoded (auto): {} chars", text.len());
                    let _ = self
                        .proxy
                        .event_tx
                        .send(RdpClientEvent::ClipboardText(text));
                } else if let Ok(text) = String::from_utf8(data.to_vec()) {
                    debug!("Clipboard text decoded (UTF-8): {} chars", text.len());
                    let _ = self
                        .proxy
                        .event_tx
                        .send(RdpClientEvent::ClipboardText(text));
                } else {
                    warn!("Failed to decode clipboard data");
                }
            }
        }
    }

    fn on_file_contents_request(&mut self, request: FileContentsRequest) {
        debug!(?request, "File contents request (not implemented)");
    }

    fn on_file_contents_response(&mut self, response: FileContentsResponse<'_>) {
        debug!(?response, "File contents response (not implemented)");
    }

    fn on_lock(&mut self, data_id: LockDataId) {
        debug!(?data_id, "Clipboard lock");
    }

    fn on_unlock(&mut self, data_id: LockDataId) {
        debug!(?data_id, "Clipboard unlock");
    }
}

/// Converts UTF-16LE bytes to a Rust String
fn string_from_utf16(data: &[u8]) -> Result<String, std::string::FromUtf16Error> {
    let u16_data: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|&c| c != 0)
        .collect();

    String::from_utf16(&u16_data)
}

/// Converts a Rust String to UTF-16LE bytes with null terminator
#[must_use]
pub fn string_to_utf16(text: &str) -> Vec<u8> {
    let mut result: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    result.extend_from_slice(&[0, 0]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_from_utf16() {
        let data = [
            0x48, 0x00, // H
            0x65, 0x00, // e
            0x6C, 0x00, // l
            0x6C, 0x00, // l
            0x6F, 0x00, // o
            0x00, 0x00, // null
        ];
        let result = string_from_utf16(&data).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_string_to_utf16() {
        let text = "Hi";
        let result = string_to_utf16(text);
        assert_eq!(result, vec![0x48, 0x00, 0x69, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_clipboard_format_info() {
        let format = ClipboardFormatInfo::unicode_text();
        assert!(format.is_text());
        assert_eq!(format.id, ClipboardFormatInfo::UNICODE_TEXT);
    }
}
