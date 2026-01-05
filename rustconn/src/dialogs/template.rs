//! Template dialog for creating and editing connection templates
//!
//! Provides a GTK4 dialog for managing connection templates, including:
//! - Creating new templates
//! - Editing existing templates
//! - Listing templates by protocol
//! - Protocol-specific configuration tabs
//!
//! Updated for GTK 4.10+ compatibility using Window instead of Dialog.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, Grid, Label, ListBox,
    ListBoxRow, Notebook, Orientation, ScrolledWindow, SpinButton, Stack, StringList,
};
use libadwaita as adw;
use adw::prelude::*;
use rustconn_core::models::{
    AwsSsmConfig, AzureBastionConfig, AzureSshConfig, BoundaryConfig, CloudflareAccessConfig,
    ConnectionTemplate, GcpIapConfig, GenericZeroTrustConfig, OciBastionConfig, ProtocolConfig,
    ProtocolType, RdpClientMode, RdpConfig, Resolution, SpiceConfig, SpiceImageCompression,
    SshAuthMethod, SshConfig, SshKeySource, TailscaleSshConfig, TeleportConfig, VncClientMode,
    VncConfig, ZeroTrustConfig, ZeroTrustProvider, ZeroTrustProviderConfig,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

/// Callback type for template dialog
pub type TemplateCallback = Rc<RefCell<Option<Box<dyn Fn(Option<ConnectionTemplate>)>>>>;

/// Template dialog for creating/editing templates
#[allow(clippy::similar_names)]
pub struct TemplateDialog {
    window: adw::Window,
    save_button: Button,
    // Basic fields
    name_entry: Entry,
    description_entry: Entry,
    protocol_dropdown: DropDown,
    host_entry: Entry,
    port_spin: SpinButton,
    username_entry: Entry,
    tags_entry: Entry,
    // Protocol stack
    protocol_stack: Stack,
    // SSH fields
    ssh_auth_dropdown: DropDown,
    ssh_key_source_dropdown: DropDown,
    ssh_key_entry: Entry,
    ssh_proxy_entry: Entry,
    ssh_identities_only: CheckButton,
    ssh_control_master: CheckButton,
    ssh_agent_forwarding: CheckButton,
    ssh_startup_entry: Entry,
    ssh_options_entry: Entry,
    // RDP fields
    rdp_client_mode_dropdown: DropDown,
    rdp_width_spin: SpinButton,
    rdp_height_spin: SpinButton,
    rdp_color_dropdown: DropDown,
    rdp_audio_check: CheckButton,
    rdp_gateway_entry: Entry,
    rdp_custom_args_entry: Entry,
    // VNC fields
    vnc_client_mode_dropdown: DropDown,
    vnc_encoding_entry: Entry,
    vnc_compression_spin: SpinButton,
    vnc_quality_spin: SpinButton,
    vnc_view_only_check: CheckButton,
    vnc_scaling_check: CheckButton,
    vnc_clipboard_check: CheckButton,
    vnc_custom_args_entry: Entry,
    // SPICE fields
    spice_tls_check: CheckButton,
    spice_ca_cert_entry: Entry,
    spice_skip_verify_check: CheckButton,
    spice_usb_check: CheckButton,
    spice_clipboard_check: CheckButton,
    spice_compression_dropdown: DropDown,
    // Zero Trust fields
    zt_provider_dropdown: DropDown,
    zt_provider_stack: Stack,
    zt_aws_target_entry: Entry,
    zt_aws_profile_entry: Entry,
    zt_aws_region_entry: Entry,
    zt_gcp_instance_entry: Entry,
    zt_gcp_zone_entry: Entry,
    zt_gcp_project_entry: Entry,
    zt_azure_bastion_resource_id_entry: Entry,
    zt_azure_bastion_rg_entry: Entry,
    zt_azure_bastion_name_entry: Entry,
    zt_azure_ssh_vm_entry: Entry,
    zt_azure_ssh_rg_entry: Entry,
    zt_oci_bastion_id_entry: Entry,
    zt_oci_target_id_entry: Entry,
    zt_oci_target_ip_entry: Entry,
    zt_cf_hostname_entry: Entry,
    zt_teleport_host_entry: Entry,
    zt_teleport_cluster_entry: Entry,
    zt_tailscale_host_entry: Entry,
    zt_boundary_target_entry: Entry,
    zt_boundary_addr_entry: Entry,
    zt_generic_command_entry: Entry,
    zt_custom_args_entry: Entry,
    // State
    editing_id: Rc<RefCell<Option<Uuid>>>,
    // Callback
    on_save: TemplateCallback,
}

impl TemplateDialog {
    /// Creates a new template dialog
    #[must_use]
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub fn new(parent: Option<&gtk4::Window>) -> Self {
        let window = adw::Window::builder()
            .title("New Template")
            .modal(true)
            .default_width(750)
            .default_height(550)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        // Create header bar with Close/Create buttons (GNOME HIG)
        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);
        let close_btn = Button::builder().label("Close").build();
        let save_btn = Button::builder()
            .label("Create")
            .css_classes(["suggested-action"])
            .build();
        header.pack_start(&close_btn);
        header.pack_end(&save_btn);

        // Close button handler
        let window_clone = window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        // Create main content area
        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        // Use ToolbarView for adw::Window
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);
        main_box.append(&content);
        window.set_content(Some(&main_box));

        // Create notebook for tabs
        let notebook = Notebook::new();
        notebook.set_vexpand(true);
        content.append(&notebook);

        // === Basic Tab ===
        let (
            basic_grid,
            name_entry,
            description_entry,
            protocol_dropdown,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
        ) = Self::create_basic_tab();
        notebook.append_page(&basic_grid, Some(&Label::new(Some("Basic"))));

        // === Protocol Tab ===
        let protocol_stack = Stack::new();
        let protocol_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&protocol_stack)
            .build();
        notebook.append_page(&protocol_scrolled, Some(&Label::new(Some("Protocol"))));

        // SSH options
        let (
            ssh_box,
            ssh_auth_dropdown,
            ssh_key_source_dropdown,
            ssh_key_entry,
            ssh_proxy_entry,
            ssh_identities_only,
            ssh_control_master,
            ssh_agent_forwarding,
            ssh_startup_entry,
            ssh_options_entry,
        ) = Self::create_ssh_options();
        protocol_stack.add_named(&ssh_box, Some("ssh"));

        // RDP options
        let (
            rdp_box,
            rdp_client_mode_dropdown,
            rdp_width_spin,
            rdp_height_spin,
            rdp_color_dropdown,
            rdp_audio_check,
            rdp_gateway_entry,
            rdp_custom_args_entry,
        ) = Self::create_rdp_options();
        protocol_stack.add_named(&rdp_box, Some("rdp"));

        // VNC options
        let (
            vnc_box,
            vnc_client_mode_dropdown,
            vnc_encoding_entry,
            vnc_compression_spin,
            vnc_quality_spin,
            vnc_view_only_check,
            vnc_scaling_check,
            vnc_clipboard_check,
            vnc_custom_args_entry,
        ) = Self::create_vnc_options();
        protocol_stack.add_named(&vnc_box, Some("vnc"));

        // SPICE options
        let (
            spice_box,
            spice_tls_check,
            spice_ca_cert_entry,
            spice_skip_verify_check,
            spice_usb_check,
            spice_clipboard_check,
            spice_compression_dropdown,
        ) = Self::create_spice_options();
        protocol_stack.add_named(&spice_box, Some("spice"));

        // Zero Trust options
        let (
            zt_box,
            zt_provider_dropdown,
            zt_provider_stack,
            zt_aws_target_entry,
            zt_aws_profile_entry,
            zt_aws_region_entry,
            zt_gcp_instance_entry,
            zt_gcp_zone_entry,
            zt_gcp_project_entry,
            zt_azure_bastion_resource_id_entry,
            zt_azure_bastion_rg_entry,
            zt_azure_bastion_name_entry,
            zt_azure_ssh_vm_entry,
            zt_azure_ssh_rg_entry,
            zt_oci_bastion_id_entry,
            zt_oci_target_id_entry,
            zt_oci_target_ip_entry,
            zt_cf_hostname_entry,
            zt_teleport_host_entry,
            zt_teleport_cluster_entry,
            zt_tailscale_host_entry,
            zt_boundary_target_entry,
            zt_boundary_addr_entry,
            zt_generic_command_entry,
            zt_custom_args_entry,
        ) = Self::create_zerotrust_options();
        protocol_stack.add_named(&zt_box, Some("zerotrust"));

        // Set initial protocol view
        protocol_stack.set_visible_child_name("ssh");

        // Connect protocol dropdown to stack and port
        let stack_clone = protocol_stack.clone();
        let port_clone = port_spin.clone();
        protocol_dropdown.connect_selected_notify(move |dropdown| {
            let protocols = ["ssh", "rdp", "vnc", "spice", "zerotrust"];
            let selected = dropdown.selected() as usize;
            if selected < protocols.len() {
                stack_clone.set_visible_child_name(protocols[selected]);
                let default_port = match selected {
                    1 => 3389.0,
                    2 | 3 => 5900.0,
                    _ => 22.0,
                };
                let current = port_clone.value();
                if (current - 22.0).abs() < 0.5
                    || (current - 3389.0).abs() < 0.5
                    || (current - 5900.0).abs() < 0.5
                {
                    port_clone.set_value(default_port);
                }
            }
        });

        let on_save: TemplateCallback = Rc::new(RefCell::new(None));
        let editing_id: Rc<RefCell<Option<Uuid>>> = Rc::new(RefCell::new(None));

        // Connect save button
        Self::connect_save_button(
            &save_btn,
            &window,
            &on_save,
            &editing_id,
            &name_entry,
            &description_entry,
            &protocol_dropdown,
            &host_entry,
            &port_spin,
            &username_entry,
            &tags_entry,
            &ssh_auth_dropdown,
            &ssh_key_source_dropdown,
            &ssh_key_entry,
            &ssh_proxy_entry,
            &ssh_identities_only,
            &ssh_control_master,
            &ssh_agent_forwarding,
            &ssh_startup_entry,
            &ssh_options_entry,
            &rdp_client_mode_dropdown,
            &rdp_width_spin,
            &rdp_height_spin,
            &rdp_color_dropdown,
            &rdp_audio_check,
            &rdp_gateway_entry,
            &rdp_custom_args_entry,
            &vnc_client_mode_dropdown,
            &vnc_encoding_entry,
            &vnc_compression_spin,
            &vnc_quality_spin,
            &vnc_view_only_check,
            &vnc_scaling_check,
            &vnc_clipboard_check,
            &vnc_custom_args_entry,
            &spice_tls_check,
            &spice_ca_cert_entry,
            &spice_skip_verify_check,
            &spice_usb_check,
            &spice_clipboard_check,
            &spice_compression_dropdown,
            &zt_provider_dropdown,
            &zt_aws_target_entry,
            &zt_aws_profile_entry,
            &zt_aws_region_entry,
            &zt_gcp_instance_entry,
            &zt_gcp_zone_entry,
            &zt_gcp_project_entry,
            &zt_azure_bastion_resource_id_entry,
            &zt_azure_bastion_rg_entry,
            &zt_azure_bastion_name_entry,
            &zt_azure_ssh_vm_entry,
            &zt_azure_ssh_rg_entry,
            &zt_oci_bastion_id_entry,
            &zt_oci_target_id_entry,
            &zt_oci_target_ip_entry,
            &zt_cf_hostname_entry,
            &zt_teleport_host_entry,
            &zt_teleport_cluster_entry,
            &zt_tailscale_host_entry,
            &zt_boundary_target_entry,
            &zt_boundary_addr_entry,
            &zt_generic_command_entry,
            &zt_custom_args_entry,
        );

        Self {
            window,
            save_button: save_btn,
            name_entry,
            description_entry,
            protocol_dropdown,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
            protocol_stack,
            ssh_auth_dropdown,
            ssh_key_source_dropdown,
            ssh_key_entry,
            ssh_proxy_entry,
            ssh_identities_only,
            ssh_control_master,
            ssh_agent_forwarding,
            ssh_startup_entry,
            ssh_options_entry,
            rdp_client_mode_dropdown,
            rdp_width_spin,
            rdp_height_spin,
            rdp_color_dropdown,
            rdp_audio_check,
            rdp_gateway_entry,
            rdp_custom_args_entry,
            vnc_client_mode_dropdown,
            vnc_encoding_entry,
            vnc_compression_spin,
            vnc_quality_spin,
            vnc_view_only_check,
            vnc_scaling_check,
            vnc_clipboard_check,
            vnc_custom_args_entry,
            spice_tls_check,
            spice_ca_cert_entry,
            spice_skip_verify_check,
            spice_usb_check,
            spice_clipboard_check,
            spice_compression_dropdown,
            zt_provider_dropdown,
            zt_provider_stack,
            zt_aws_target_entry,
            zt_aws_profile_entry,
            zt_aws_region_entry,
            zt_gcp_instance_entry,
            zt_gcp_zone_entry,
            zt_gcp_project_entry,
            zt_azure_bastion_resource_id_entry,
            zt_azure_bastion_rg_entry,
            zt_azure_bastion_name_entry,
            zt_azure_ssh_vm_entry,
            zt_azure_ssh_rg_entry,
            zt_oci_bastion_id_entry,
            zt_oci_target_id_entry,
            zt_oci_target_ip_entry,
            zt_cf_hostname_entry,
            zt_teleport_host_entry,
            zt_teleport_cluster_entry,
            zt_tailscale_host_entry,
            zt_boundary_target_entry,
            zt_boundary_addr_entry,
            zt_generic_command_entry,
            zt_custom_args_entry,
            editing_id,
            on_save,
        }
    }

    fn create_basic_tab() -> (
        ScrolledWindow,
        Entry,
        Entry,
        DropDown,
        Entry,
        SpinButton,
        Entry,
        Entry,
    ) {
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let grid = Grid::builder()
            .row_spacing(8)
            .column_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let mut row = 0;

        let name_label = Label::builder()
            .label("Name:")
            .halign(gtk4::Align::End)
            .build();
        let name_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Template name")
            .build();
        grid.attach(&name_label, 0, row, 1, 1);
        grid.attach(&name_entry, 1, row, 1, 1);
        row += 1;

        let desc_label = Label::builder()
            .label("Description:")
            .halign(gtk4::Align::End)
            .build();
        let description_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Optional description")
            .build();
        grid.attach(&desc_label, 0, row, 1, 1);
        grid.attach(&description_entry, 1, row, 1, 1);
        row += 1;

        let protocol_label = Label::builder()
            .label("Protocol:")
            .halign(gtk4::Align::End)
            .build();
        let protocols = StringList::new(&["SSH", "RDP", "VNC", "SPICE", "ZeroTrust"]);
        let protocol_dropdown = DropDown::builder().model(&protocols).hexpand(true).build();
        grid.attach(&protocol_label, 0, row, 1, 1);
        grid.attach(&protocol_dropdown, 1, row, 1, 1);
        row += 1;

        let host_label = Label::builder()
            .label("Default Host:")
            .halign(gtk4::Align::End)
            .build();
        let host_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Leave empty for user to fill in")
            .build();
        grid.attach(&host_label, 0, row, 1, 1);
        grid.attach(&host_entry, 1, row, 1, 1);
        row += 1;

        let port_label = Label::builder()
            .label("Default Port:")
            .halign(gtk4::Align::End)
            .build();
        let port_spin = SpinButton::with_range(1.0, 65535.0, 1.0);
        port_spin.set_value(22.0);
        grid.attach(&port_label, 0, row, 1, 1);
        grid.attach(&port_spin, 1, row, 1, 1);
        row += 1;

        let user_label = Label::builder()
            .label("Default Username:")
            .halign(gtk4::Align::End)
            .build();
        let username_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Optional default username")
            .build();
        grid.attach(&user_label, 0, row, 1, 1);
        grid.attach(&username_entry, 1, row, 1, 1);
        row += 1;

        let tags_label = Label::builder()
            .label("Default Tags:")
            .halign(gtk4::Align::End)
            .build();
        let tags_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("tag1, tag2, ...")
            .build();
        grid.attach(&tags_label, 0, row, 1, 1);
        grid.attach(&tags_entry, 1, row, 1, 1);

        scrolled.set_child(Some(&grid));
        (
            scrolled,
            name_entry,
            description_entry,
            protocol_dropdown,
            host_entry,
            port_spin,
            username_entry,
            tags_entry,
        )
    }

    #[allow(clippy::type_complexity)]
    fn create_ssh_options() -> (
        GtkBox,
        DropDown,
        DropDown,
        Entry,
        Entry,
        CheckButton,
        CheckButton,
        CheckButton,
        Entry,
        Entry,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        let auth_label = Label::builder()
            .label("Auth Method:")
            .halign(gtk4::Align::End)
            .build();
        let auth_list = StringList::new(&[
            "Password",
            "Public Key",
            "Keyboard Interactive",
            "SSH Agent",
        ]);
        let auth_dropdown = DropDown::new(Some(auth_list), gtk4::Expression::NONE);
        auth_dropdown.set_selected(0);
        grid.attach(&auth_label, 0, row, 1, 1);
        grid.attach(&auth_dropdown, 1, row, 2, 1);
        row += 1;

        let key_source_label = Label::builder()
            .label("Key Source:")
            .halign(gtk4::Align::End)
            .build();
        let key_source_list = StringList::new(&["Default", "File", "Agent"]);
        let key_source_dropdown = DropDown::new(Some(key_source_list), gtk4::Expression::NONE);
        key_source_dropdown.set_selected(0);
        grid.attach(&key_source_label, 0, row, 1, 1);
        grid.attach(&key_source_dropdown, 1, row, 2, 1);
        row += 1;

        let key_label = Label::builder()
            .label("Key File:")
            .halign(gtk4::Align::End)
            .build();
        let key_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Path to SSH key")
            .build();
        grid.attach(&key_label, 0, row, 1, 1);
        grid.attach(&key_entry, 1, row, 2, 1);
        row += 1;

        let proxy_label = Label::builder()
            .label("ProxyJump:")
            .halign(gtk4::Align::End)
            .build();
        let proxy_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("user@jumphost")
            .build();
        grid.attach(&proxy_label, 0, row, 1, 1);
        grid.attach(&proxy_entry, 1, row, 2, 1);
        row += 1;

        let identities_only = CheckButton::builder()
            .label("Use only specified key (IdentitiesOnly)")
            .build();
        grid.attach(&identities_only, 1, row, 2, 1);
        row += 1;

        let control_master = CheckButton::builder()
            .label("Enable ControlMaster (connection multiplexing)")
            .build();
        grid.attach(&control_master, 1, row, 2, 1);
        row += 1;

        let agent_forwarding = CheckButton::builder()
            .label("Enable Agent Forwarding (-A)")
            .build();
        grid.attach(&agent_forwarding, 1, row, 2, 1);
        row += 1;

        let startup_label = Label::builder()
            .label("Startup Command:")
            .halign(gtk4::Align::End)
            .build();
        let startup_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Command to run on connect")
            .build();
        grid.attach(&startup_label, 0, row, 1, 1);
        grid.attach(&startup_entry, 1, row, 2, 1);
        row += 1;

        let options_label = Label::builder()
            .label("Custom Options:")
            .halign(gtk4::Align::End)
            .build();
        let options_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Key=Value, Key2=Value2")
            .build();
        grid.attach(&options_label, 0, row, 1, 1);
        grid.attach(&options_entry, 1, row, 2, 1);

        (
            vbox,
            auth_dropdown,
            key_source_dropdown,
            key_entry,
            proxy_entry,
            identities_only,
            control_master,
            agent_forwarding,
            startup_entry,
            options_entry,
        )
    }

    #[allow(clippy::type_complexity)]
    fn create_rdp_options() -> (
        GtkBox,
        DropDown,
        SpinButton,
        SpinButton,
        DropDown,
        CheckButton,
        Entry,
        Entry,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        let client_mode_label = Label::builder()
            .label("Client mode:")
            .halign(gtk4::Align::End)
            .build();
        let client_mode_list = StringList::new(&[
            RdpClientMode::Embedded.display_name(),
            RdpClientMode::External.display_name(),
        ]);
        let client_mode_dropdown = DropDown::builder()
            .model(&client_mode_list)
            .hexpand(true)
            .build();
        grid.attach(&client_mode_label, 0, row, 1, 1);
        grid.attach(&client_mode_dropdown, 1, row, 2, 1);
        row += 1;

        let res_label = Label::builder()
            .label("Resolution:")
            .halign(gtk4::Align::End)
            .build();
        let res_hbox = GtkBox::new(Orientation::Horizontal, 4);
        let width_adj = gtk4::Adjustment::new(1920.0, 640.0, 7680.0, 1.0, 100.0, 0.0);
        let width_spin = SpinButton::builder()
            .adjustment(&width_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        let x_label = Label::new(Some("x"));
        let height_adj = gtk4::Adjustment::new(1080.0, 480.0, 4320.0, 1.0, 100.0, 0.0);
        let height_spin = SpinButton::builder()
            .adjustment(&height_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        res_hbox.append(&width_spin);
        res_hbox.append(&x_label);
        res_hbox.append(&height_spin);
        grid.attach(&res_label, 0, row, 1, 1);
        grid.attach(&res_hbox, 1, row, 2, 1);
        row += 1;

        let color_label = Label::builder()
            .label("Color Depth:")
            .halign(gtk4::Align::End)
            .build();
        let color_list = StringList::new(&["32-bit", "24-bit", "16-bit", "15-bit", "8-bit"]);
        let color_dropdown = DropDown::new(Some(color_list), gtk4::Expression::NONE);
        color_dropdown.set_selected(0);
        grid.attach(&color_label, 0, row, 1, 1);
        grid.attach(&color_dropdown, 1, row, 2, 1);
        row += 1;

        let audio_check = CheckButton::builder()
            .label("Enable audio redirection")
            .build();
        grid.attach(&audio_check, 1, row, 2, 1);
        row += 1;

        let gateway_label = Label::builder()
            .label("RDP Gateway:")
            .halign(gtk4::Align::End)
            .build();
        let gateway_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("gateway.example.com")
            .build();
        grid.attach(&gateway_label, 0, row, 1, 1);
        grid.attach(&gateway_entry, 1, row, 2, 1);
        row += 1;

        let args_label = Label::builder()
            .label("Custom Args:")
            .halign(gtk4::Align::End)
            .build();
        let custom_args_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Additional command-line arguments")
            .build();
        grid.attach(&args_label, 0, row, 1, 1);
        grid.attach(&custom_args_entry, 1, row, 2, 1);

        (
            vbox,
            client_mode_dropdown,
            width_spin,
            height_spin,
            color_dropdown,
            audio_check,
            gateway_entry,
            custom_args_entry,
        )
    }

    #[allow(clippy::type_complexity)]
    fn create_vnc_options() -> (
        GtkBox,
        DropDown,
        Entry,
        SpinButton,
        SpinButton,
        CheckButton,
        CheckButton,
        CheckButton,
        Entry,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        let client_mode_label = Label::builder()
            .label("Client mode:")
            .halign(gtk4::Align::End)
            .build();
        let client_mode_list = StringList::new(&[
            VncClientMode::Embedded.display_name(),
            VncClientMode::External.display_name(),
        ]);
        let client_mode_dropdown = DropDown::builder()
            .model(&client_mode_list)
            .hexpand(true)
            .build();
        grid.attach(&client_mode_label, 0, row, 1, 1);
        grid.attach(&client_mode_dropdown, 1, row, 2, 1);
        row += 1;

        let encoding_label = Label::builder()
            .label("Encoding:")
            .halign(gtk4::Align::End)
            .build();
        let encoding_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("tight, zrle, hextile")
            .build();
        grid.attach(&encoding_label, 0, row, 1, 1);
        grid.attach(&encoding_entry, 1, row, 2, 1);
        row += 1;

        let compression_label = Label::builder()
            .label("Compression:")
            .halign(gtk4::Align::End)
            .build();
        let compression_adj = gtk4::Adjustment::new(6.0, 0.0, 9.0, 1.0, 1.0, 0.0);
        let compression_spin = SpinButton::builder()
            .adjustment(&compression_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&compression_label, 0, row, 1, 1);
        grid.attach(&compression_spin, 1, row, 1, 1);
        row += 1;

        let quality_label = Label::builder()
            .label("Quality:")
            .halign(gtk4::Align::End)
            .build();
        let quality_adj = gtk4::Adjustment::new(6.0, 0.0, 9.0, 1.0, 1.0, 0.0);
        let quality_spin = SpinButton::builder()
            .adjustment(&quality_adj)
            .climb_rate(1.0)
            .digits(0)
            .build();
        grid.attach(&quality_label, 0, row, 1, 1);
        grid.attach(&quality_spin, 1, row, 1, 1);
        row += 1;

        let view_only_check = CheckButton::builder()
            .label("View-only mode (no input)")
            .build();
        grid.attach(&view_only_check, 1, row, 2, 1);
        row += 1;

        let scaling_check = CheckButton::builder()
            .label("Scale display to fit window")
            .active(true)
            .build();
        grid.attach(&scaling_check, 1, row, 2, 1);
        row += 1;

        let clipboard_check = CheckButton::builder()
            .label("Enable clipboard sharing")
            .active(true)
            .build();
        grid.attach(&clipboard_check, 1, row, 2, 1);
        row += 1;

        let args_label = Label::builder()
            .label("Custom args:")
            .halign(gtk4::Align::End)
            .build();
        let custom_args_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Additional arguments")
            .build();
        grid.attach(&args_label, 0, row, 1, 1);
        grid.attach(&custom_args_entry, 1, row, 2, 1);

        (
            vbox,
            client_mode_dropdown,
            encoding_entry,
            compression_spin,
            quality_spin,
            view_only_check,
            scaling_check,
            clipboard_check,
            custom_args_entry,
        )
    }

    #[allow(clippy::type_complexity)]
    fn create_spice_options() -> (
        GtkBox,
        CheckButton,
        Entry,
        CheckButton,
        CheckButton,
        CheckButton,
        DropDown,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let mut row = 0;

        let tls_check = CheckButton::builder()
            .label("Enable TLS encryption")
            .build();
        grid.attach(&tls_check, 1, row, 2, 1);
        row += 1;

        let ca_cert_label = Label::builder()
            .label("CA Certificate:")
            .halign(gtk4::Align::End)
            .build();
        let ca_cert_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Path to CA certificate (optional)")
            .build();
        grid.attach(&ca_cert_label, 0, row, 1, 1);
        grid.attach(&ca_cert_entry, 1, row, 2, 1);
        row += 1;

        let skip_verify_check = CheckButton::builder()
            .label("Skip certificate verification (insecure)")
            .build();
        grid.attach(&skip_verify_check, 1, row, 2, 1);
        row += 1;

        let usb_check = CheckButton::builder()
            .label("Enable USB redirection")
            .build();
        grid.attach(&usb_check, 1, row, 2, 1);
        row += 1;

        let clipboard_check = CheckButton::builder()
            .label("Enable clipboard sharing")
            .active(true)
            .build();
        grid.attach(&clipboard_check, 1, row, 2, 1);
        row += 1;

        let compression_label = Label::builder()
            .label("Image Compression:")
            .halign(gtk4::Align::End)
            .build();
        let compression_list = StringList::new(&["Auto", "Off", "GLZ", "LZ", "QUIC"]);
        let compression_dropdown = DropDown::new(Some(compression_list), gtk4::Expression::NONE);
        compression_dropdown.set_selected(0);
        grid.attach(&compression_label, 0, row, 1, 1);
        grid.attach(&compression_dropdown, 1, row, 2, 1);

        (
            vbox,
            tls_check,
            ca_cert_entry,
            skip_verify_check,
            usb_check,
            clipboard_check,
            compression_dropdown,
        )
    }

    #[allow(clippy::type_complexity, clippy::too_many_lines, clippy::similar_names)]
    fn create_zerotrust_options() -> (
        GtkBox,
        DropDown,
        Stack,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
        Entry,
    ) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);

        let provider_hbox = GtkBox::new(Orientation::Horizontal, 8);
        let provider_label = Label::builder()
            .label("Provider:")
            .halign(gtk4::Align::End)
            .build();
        let provider_list = StringList::new(&[
            "AWS Session Manager",
            "GCP IAP Tunnel",
            "Azure Bastion",
            "Azure SSH (AAD)",
            "OCI Bastion",
            "Cloudflare Access",
            "Teleport",
            "Tailscale SSH",
            "HashiCorp Boundary",
            "Generic Command",
        ]);
        let provider_dropdown = DropDown::new(Some(provider_list), gtk4::Expression::NONE);
        provider_dropdown.set_selected(0);
        provider_hbox.append(&provider_label);
        provider_hbox.append(&provider_dropdown);
        vbox.append(&provider_hbox);

        let provider_stack = Stack::new();
        provider_stack.set_vexpand(true);
        vbox.append(&provider_stack);

        // AWS SSM
        let (aws_box, aws_target, aws_profile, aws_region) = Self::create_aws_fields();
        provider_stack.add_named(&aws_box, Some("aws_ssm"));

        // GCP IAP
        let (gcp_box, gcp_instance, gcp_zone, gcp_project) = Self::create_gcp_fields();
        provider_stack.add_named(&gcp_box, Some("gcp_iap"));

        // Azure Bastion
        let (azure_bastion_box, azure_bastion_resource_id, azure_bastion_rg, azure_bastion_name) =
            Self::create_azure_bastion_fields();
        provider_stack.add_named(&azure_bastion_box, Some("azure_bastion"));

        // Azure SSH
        let (azure_ssh_box, azure_ssh_vm, azure_ssh_rg) = Self::create_azure_ssh_fields();
        provider_stack.add_named(&azure_ssh_box, Some("azure_ssh"));

        // OCI Bastion
        let (oci_box, oci_bastion_id, oci_target_id, oci_target_ip) = Self::create_oci_fields();
        provider_stack.add_named(&oci_box, Some("oci_bastion"));

        // Cloudflare
        let (cf_box, cf_hostname) = Self::create_cloudflare_fields();
        provider_stack.add_named(&cf_box, Some("cloudflare"));

        // Teleport
        let (teleport_box, teleport_host, teleport_cluster) = Self::create_teleport_fields();
        provider_stack.add_named(&teleport_box, Some("teleport"));

        // Tailscale
        let (tailscale_box, tailscale_host) = Self::create_tailscale_fields();
        provider_stack.add_named(&tailscale_box, Some("tailscale"));

        // Boundary
        let (boundary_box, boundary_target, boundary_addr) = Self::create_boundary_fields();
        provider_stack.add_named(&boundary_box, Some("boundary"));

        // Generic
        let (generic_box, generic_command) = Self::create_generic_fields();
        provider_stack.add_named(&generic_box, Some("generic"));

        provider_stack.set_visible_child_name("aws_ssm");

        let stack_clone = provider_stack.clone();
        provider_dropdown.connect_selected_notify(move |dropdown| {
            let providers = [
                "aws_ssm",
                "gcp_iap",
                "azure_bastion",
                "azure_ssh",
                "oci_bastion",
                "cloudflare",
                "teleport",
                "tailscale",
                "boundary",
                "generic",
            ];
            let selected = dropdown.selected() as usize;
            if selected < providers.len() {
                stack_clone.set_visible_child_name(providers[selected]);
            }
        });

        let custom_args_hbox = GtkBox::new(Orientation::Horizontal, 8);
        let custom_args_label = Label::builder()
            .label("Custom Args:")
            .halign(gtk4::Align::End)
            .build();
        let custom_args_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("Additional command-line arguments")
            .build();
        custom_args_hbox.append(&custom_args_label);
        custom_args_hbox.append(&custom_args_entry);
        vbox.append(&custom_args_hbox);

        (
            vbox,
            provider_dropdown,
            provider_stack,
            aws_target,
            aws_profile,
            aws_region,
            gcp_instance,
            gcp_zone,
            gcp_project,
            azure_bastion_resource_id,
            azure_bastion_rg,
            azure_bastion_name,
            azure_ssh_vm,
            azure_ssh_rg,
            oci_bastion_id,
            oci_target_id,
            oci_target_ip,
            cf_hostname,
            teleport_host,
            teleport_cluster,
            tailscale_host,
            boundary_target,
            boundary_addr,
            generic_command,
            custom_args_entry,
        )
    }

    fn create_aws_fields() -> (GtkBox, Entry, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let target_label = Label::builder()
            .label("Instance ID:")
            .halign(gtk4::Align::End)
            .build();
        let target_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("i-0123456789abcdef0")
            .build();
        grid.attach(&target_label, 0, 0, 1, 1);
        grid.attach(&target_entry, 1, 0, 1, 1);

        let profile_label = Label::builder()
            .label("AWS Profile:")
            .halign(gtk4::Align::End)
            .build();
        let profile_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("default")
            .text("default")
            .build();
        grid.attach(&profile_label, 0, 1, 1, 1);
        grid.attach(&profile_entry, 1, 1, 1, 1);

        let region_label = Label::builder()
            .label("Region:")
            .halign(gtk4::Align::End)
            .build();
        let region_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("us-east-1 (optional)")
            .build();
        grid.attach(&region_label, 0, 2, 1, 1);
        grid.attach(&region_entry, 1, 2, 1, 1);

        (vbox, target_entry, profile_entry, region_entry)
    }

    fn create_gcp_fields() -> (GtkBox, Entry, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let instance_label = Label::builder()
            .label("Instance:")
            .halign(gtk4::Align::End)
            .build();
        let instance_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("my-instance")
            .build();
        grid.attach(&instance_label, 0, 0, 1, 1);
        grid.attach(&instance_entry, 1, 0, 1, 1);

        let zone_label = Label::builder()
            .label("Zone:")
            .halign(gtk4::Align::End)
            .build();
        let zone_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("us-central1-a")
            .build();
        grid.attach(&zone_label, 0, 1, 1, 1);
        grid.attach(&zone_entry, 1, 1, 1, 1);

        let project_label = Label::builder()
            .label("Project:")
            .halign(gtk4::Align::End)
            .build();
        let project_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("my-project (optional)")
            .build();
        grid.attach(&project_label, 0, 2, 1, 1);
        grid.attach(&project_entry, 1, 2, 1, 1);

        (vbox, instance_entry, zone_entry, project_entry)
    }

    fn create_azure_bastion_fields() -> (GtkBox, Entry, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let resource_id_label = Label::builder()
            .label("Target Resource ID:")
            .halign(gtk4::Align::End)
            .build();
        let resource_id_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("/subscriptions/...")
            .build();
        grid.attach(&resource_id_label, 0, 0, 1, 1);
        grid.attach(&resource_id_entry, 1, 0, 1, 1);

        let rg_label = Label::builder()
            .label("Resource Group:")
            .halign(gtk4::Align::End)
            .build();
        let rg_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("my-resource-group")
            .build();
        grid.attach(&rg_label, 0, 1, 1, 1);
        grid.attach(&rg_entry, 1, 1, 1, 1);

        let bastion_label = Label::builder()
            .label("Bastion Name:")
            .halign(gtk4::Align::End)
            .build();
        let bastion_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("my-bastion")
            .build();
        grid.attach(&bastion_label, 0, 2, 1, 1);
        grid.attach(&bastion_entry, 1, 2, 1, 1);

        (vbox, resource_id_entry, rg_entry, bastion_entry)
    }

    fn create_azure_ssh_fields() -> (GtkBox, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let vm_label = Label::builder()
            .label("VM Name:")
            .halign(gtk4::Align::End)
            .build();
        let vm_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("my-vm")
            .build();
        grid.attach(&vm_label, 0, 0, 1, 1);
        grid.attach(&vm_entry, 1, 0, 1, 1);

        let rg_label = Label::builder()
            .label("Resource Group:")
            .halign(gtk4::Align::End)
            .build();
        let rg_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("my-resource-group")
            .build();
        grid.attach(&rg_label, 0, 1, 1, 1);
        grid.attach(&rg_entry, 1, 1, 1, 1);

        (vbox, vm_entry, rg_entry)
    }

    #[allow(clippy::similar_names)]
    fn create_oci_fields() -> (GtkBox, Entry, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let bastion_id_label = Label::builder()
            .label("Bastion OCID:")
            .halign(gtk4::Align::End)
            .build();
        let bastion_id_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("ocid1.bastion...")
            .build();
        grid.attach(&bastion_id_label, 0, 0, 1, 1);
        grid.attach(&bastion_id_entry, 1, 0, 1, 1);

        let target_id_label = Label::builder()
            .label("Target OCID:")
            .halign(gtk4::Align::End)
            .build();
        let target_id_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("ocid1.instance...")
            .build();
        grid.attach(&target_id_label, 0, 1, 1, 1);
        grid.attach(&target_id_entry, 1, 1, 1, 1);

        let target_ip_label = Label::builder()
            .label("Target IP:")
            .halign(gtk4::Align::End)
            .build();
        let target_ip_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("10.0.0.1")
            .build();
        grid.attach(&target_ip_label, 0, 2, 1, 1);
        grid.attach(&target_ip_entry, 1, 2, 1, 1);

        (vbox, bastion_id_entry, target_id_entry, target_ip_entry)
    }

    fn create_cloudflare_fields() -> (GtkBox, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let hostname_label = Label::builder()
            .label("Hostname:")
            .halign(gtk4::Align::End)
            .build();
        let hostname_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("ssh.example.com")
            .build();
        grid.attach(&hostname_label, 0, 0, 1, 1);
        grid.attach(&hostname_entry, 1, 0, 1, 1);

        (vbox, hostname_entry)
    }

    fn create_teleport_fields() -> (GtkBox, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let host_label = Label::builder()
            .label("Host:")
            .halign(gtk4::Align::End)
            .build();
        let host_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("node-name")
            .build();
        grid.attach(&host_label, 0, 0, 1, 1);
        grid.attach(&host_entry, 1, 0, 1, 1);

        let cluster_label = Label::builder()
            .label("Cluster:")
            .halign(gtk4::Align::End)
            .build();
        let cluster_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("teleport.example.com")
            .build();
        grid.attach(&cluster_label, 0, 1, 1, 1);
        grid.attach(&cluster_entry, 1, 1, 1, 1);

        (vbox, host_entry, cluster_entry)
    }

    fn create_tailscale_fields() -> (GtkBox, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let host_label = Label::builder()
            .label("Tailscale Host:")
            .halign(gtk4::Align::End)
            .build();
        let host_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("hostname or IP")
            .build();
        grid.attach(&host_label, 0, 0, 1, 1);
        grid.attach(&host_entry, 1, 0, 1, 1);

        (vbox, host_entry)
    }

    fn create_boundary_fields() -> (GtkBox, Entry, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let target_label = Label::builder()
            .label("Target ID:")
            .halign(gtk4::Align::End)
            .build();
        let target_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("ttcp_...")
            .build();
        grid.attach(&target_label, 0, 0, 1, 1);
        grid.attach(&target_entry, 1, 0, 1, 1);

        let addr_label = Label::builder()
            .label("Boundary Address:")
            .halign(gtk4::Align::End)
            .build();
        let addr_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("https://boundary.example.com")
            .build();
        grid.attach(&addr_label, 0, 1, 1, 1);
        grid.attach(&addr_entry, 1, 1, 1, 1);

        (vbox, target_entry, addr_entry)
    }

    fn create_generic_fields() -> (GtkBox, Entry) {
        let vbox = GtkBox::new(Orientation::Vertical, 8);
        let grid = Grid::builder().row_spacing(8).column_spacing(12).build();
        vbox.append(&grid);

        let command_label = Label::builder()
            .label("Command:")
            .halign(gtk4::Align::End)
            .build();
        let command_entry = Entry::builder()
            .hexpand(true)
            .placeholder_text("ssh -o ProxyCommand=...")
            .build();
        grid.attach(&command_label, 0, 0, 1, 1);
        grid.attach(&command_entry, 1, 0, 1, 1);

        (vbox, command_entry)
    }

    #[allow(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        clippy::similar_names
    )]
    fn connect_save_button(
        save_btn: &Button,
        window: &adw::Window,
        on_save: &TemplateCallback,
        editing_id: &Rc<RefCell<Option<Uuid>>>,
        name_entry: &Entry,
        description_entry: &Entry,
        protocol_dropdown: &DropDown,
        host_entry: &Entry,
        port_spin: &SpinButton,
        username_entry: &Entry,
        tags_entry: &Entry,
        ssh_auth_dropdown: &DropDown,
        ssh_key_source_dropdown: &DropDown,
        ssh_key_entry: &Entry,
        ssh_proxy_entry: &Entry,
        ssh_identities_only: &CheckButton,
        ssh_control_master: &CheckButton,
        ssh_agent_forwarding: &CheckButton,
        ssh_startup_entry: &Entry,
        ssh_options_entry: &Entry,
        rdp_client_mode_dropdown: &DropDown,
        rdp_width_spin: &SpinButton,
        rdp_height_spin: &SpinButton,
        rdp_color_dropdown: &DropDown,
        rdp_audio_check: &CheckButton,
        rdp_gateway_entry: &Entry,
        rdp_custom_args_entry: &Entry,
        vnc_client_mode_dropdown: &DropDown,
        vnc_encoding_entry: &Entry,
        vnc_compression_spin: &SpinButton,
        vnc_quality_spin: &SpinButton,
        vnc_view_only_check: &CheckButton,
        vnc_scaling_check: &CheckButton,
        vnc_clipboard_check: &CheckButton,
        vnc_custom_args_entry: &Entry,
        spice_tls_check: &CheckButton,
        spice_ca_cert_entry: &Entry,
        spice_skip_verify_check: &CheckButton,
        spice_usb_check: &CheckButton,
        spice_clipboard_check: &CheckButton,
        spice_compression_dropdown: &DropDown,
        zt_provider_dropdown: &DropDown,
        zt_aws_target: &Entry,
        zt_aws_profile: &Entry,
        zt_aws_region: &Entry,
        zt_gcp_instance: &Entry,
        zt_gcp_zone: &Entry,
        zt_gcp_project: &Entry,
        zt_azure_bastion_resource_id: &Entry,
        zt_azure_bastion_rg: &Entry,
        zt_azure_bastion_name: &Entry,
        zt_azure_ssh_vm: &Entry,
        zt_azure_ssh_rg: &Entry,
        zt_oci_bastion_id: &Entry,
        zt_oci_target_id: &Entry,
        zt_oci_target_ip: &Entry,
        zt_cf_hostname: &Entry,
        zt_teleport_host: &Entry,
        zt_teleport_cluster: &Entry,
        zt_tailscale_host: &Entry,
        zt_boundary_target: &Entry,
        zt_boundary_addr: &Entry,
        zt_generic_command: &Entry,
        zt_custom_args: &Entry,
    ) {
        let window = window.clone();
        let on_save = on_save.clone();
        let editing_id = editing_id.clone();
        let name_entry = name_entry.clone();
        let description_entry = description_entry.clone();
        let protocol_dropdown = protocol_dropdown.clone();
        let host_entry = host_entry.clone();
        let port_spin = port_spin.clone();
        let username_entry = username_entry.clone();
        let tags_entry = tags_entry.clone();
        let ssh_auth_dropdown = ssh_auth_dropdown.clone();
        let ssh_key_source_dropdown = ssh_key_source_dropdown.clone();
        let ssh_key_entry = ssh_key_entry.clone();
        let ssh_proxy_entry = ssh_proxy_entry.clone();
        let ssh_identities_only = ssh_identities_only.clone();
        let ssh_control_master = ssh_control_master.clone();
        let ssh_agent_forwarding = ssh_agent_forwarding.clone();
        let ssh_startup_entry = ssh_startup_entry.clone();
        let ssh_options_entry = ssh_options_entry.clone();
        let rdp_client_mode_dropdown = rdp_client_mode_dropdown.clone();
        let rdp_width_spin = rdp_width_spin.clone();
        let rdp_height_spin = rdp_height_spin.clone();
        let rdp_color_dropdown = rdp_color_dropdown.clone();
        let rdp_audio_check = rdp_audio_check.clone();
        let rdp_gateway_entry = rdp_gateway_entry.clone();
        let rdp_custom_args_entry = rdp_custom_args_entry.clone();
        let vnc_client_mode_dropdown = vnc_client_mode_dropdown.clone();
        let vnc_encoding_entry = vnc_encoding_entry.clone();
        let vnc_compression_spin = vnc_compression_spin.clone();
        let vnc_quality_spin = vnc_quality_spin.clone();
        let vnc_view_only_check = vnc_view_only_check.clone();
        let vnc_scaling_check = vnc_scaling_check.clone();
        let vnc_clipboard_check = vnc_clipboard_check.clone();
        let vnc_custom_args_entry = vnc_custom_args_entry.clone();
        let spice_tls_check = spice_tls_check.clone();
        let spice_ca_cert_entry = spice_ca_cert_entry.clone();
        let spice_skip_verify_check = spice_skip_verify_check.clone();
        let spice_usb_check = spice_usb_check.clone();
        let spice_clipboard_check = spice_clipboard_check.clone();
        let spice_compression_dropdown = spice_compression_dropdown.clone();
        let zt_provider_dropdown = zt_provider_dropdown.clone();
        let zt_aws_target = zt_aws_target.clone();
        let zt_aws_profile = zt_aws_profile.clone();
        let zt_aws_region = zt_aws_region.clone();
        let zt_gcp_instance = zt_gcp_instance.clone();
        let zt_gcp_zone = zt_gcp_zone.clone();
        let zt_gcp_project = zt_gcp_project.clone();
        let zt_azure_bastion_resource_id = zt_azure_bastion_resource_id.clone();
        let zt_azure_bastion_rg = zt_azure_bastion_rg.clone();
        let zt_azure_bastion_name = zt_azure_bastion_name.clone();
        let zt_azure_ssh_vm = zt_azure_ssh_vm.clone();
        let zt_azure_ssh_rg = zt_azure_ssh_rg.clone();
        let zt_oci_bastion_id = zt_oci_bastion_id.clone();
        let zt_oci_target_id = zt_oci_target_id.clone();
        let zt_oci_target_ip = zt_oci_target_ip.clone();
        let zt_cf_hostname = zt_cf_hostname.clone();
        let zt_teleport_host = zt_teleport_host.clone();
        let zt_teleport_cluster = zt_teleport_cluster.clone();
        let zt_tailscale_host = zt_tailscale_host.clone();
        let zt_boundary_target = zt_boundary_target.clone();
        let zt_boundary_addr = zt_boundary_addr.clone();
        let zt_generic_command = zt_generic_command.clone();
        let zt_custom_args = zt_custom_args.clone();

        save_btn.connect_clicked(move |_| {
            let name = name_entry.text();
            if name.trim().is_empty() {
                crate::toast::show_toast_on_window(
                    &window,
                    "Template name is required",
                    crate::toast::ToastType::Warning,
                );
                return;
            }

            let protocol_idx = protocol_dropdown.selected() as usize;
            let protocol_config = Self::build_protocol_config(
                protocol_idx,
                &ssh_auth_dropdown,
                &ssh_key_source_dropdown,
                &ssh_key_entry,
                &ssh_proxy_entry,
                &ssh_identities_only,
                &ssh_control_master,
                &ssh_agent_forwarding,
                &ssh_startup_entry,
                &ssh_options_entry,
                &rdp_client_mode_dropdown,
                &rdp_width_spin,
                &rdp_height_spin,
                &rdp_color_dropdown,
                &rdp_audio_check,
                &rdp_gateway_entry,
                &rdp_custom_args_entry,
                &vnc_client_mode_dropdown,
                &vnc_encoding_entry,
                &vnc_compression_spin,
                &vnc_quality_spin,
                &vnc_view_only_check,
                &vnc_scaling_check,
                &vnc_clipboard_check,
                &vnc_custom_args_entry,
                &spice_tls_check,
                &spice_ca_cert_entry,
                &spice_skip_verify_check,
                &spice_usb_check,
                &spice_clipboard_check,
                &spice_compression_dropdown,
                &zt_provider_dropdown,
                &zt_aws_target,
                &zt_aws_profile,
                &zt_aws_region,
                &zt_gcp_instance,
                &zt_gcp_zone,
                &zt_gcp_project,
                &zt_azure_bastion_resource_id,
                &zt_azure_bastion_rg,
                &zt_azure_bastion_name,
                &zt_azure_ssh_vm,
                &zt_azure_ssh_rg,
                &zt_oci_bastion_id,
                &zt_oci_target_id,
                &zt_oci_target_ip,
                &zt_cf_hostname,
                &zt_teleport_host,
                &zt_teleport_cluster,
                &zt_tailscale_host,
                &zt_boundary_target,
                &zt_boundary_addr,
                &zt_generic_command,
                &zt_custom_args,
            );

            let mut template = ConnectionTemplate::new(name.trim().to_string(), protocol_config);

            let desc = description_entry.text();
            if !desc.trim().is_empty() {
                template.description = Some(desc.trim().to_string());
            }

            let host = host_entry.text();
            if !host.trim().is_empty() {
                template.host = host.trim().to_string();
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let port = port_spin.value() as u16;
            template.port = port;

            let username = username_entry.text();
            if !username.trim().is_empty() {
                template.username = Some(username.trim().to_string());
            }

            let tags_text = tags_entry.text();
            if !tags_text.trim().is_empty() {
                template.tags = tags_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }

            if let Some(id) = *editing_id.borrow() {
                template.id = id;
            }

            if let Some(ref cb) = *on_save.borrow() {
                cb(Some(template));
            }
            window.close();
        });
    }

    #[allow(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        clippy::similar_names
    )]
    fn build_protocol_config(
        protocol_idx: usize,
        ssh_auth_dropdown: &DropDown,
        ssh_key_source_dropdown: &DropDown,
        ssh_key_entry: &Entry,
        ssh_proxy_entry: &Entry,
        ssh_identities_only: &CheckButton,
        ssh_control_master: &CheckButton,
        ssh_agent_forwarding: &CheckButton,
        ssh_startup_entry: &Entry,
        ssh_options_entry: &Entry,
        rdp_client_mode_dropdown: &DropDown,
        rdp_width_spin: &SpinButton,
        rdp_height_spin: &SpinButton,
        rdp_color_dropdown: &DropDown,
        rdp_audio_check: &CheckButton,
        rdp_gateway_entry: &Entry,
        rdp_custom_args_entry: &Entry,
        vnc_client_mode_dropdown: &DropDown,
        vnc_encoding_entry: &Entry,
        vnc_compression_spin: &SpinButton,
        vnc_quality_spin: &SpinButton,
        vnc_view_only_check: &CheckButton,
        vnc_scaling_check: &CheckButton,
        vnc_clipboard_check: &CheckButton,
        vnc_custom_args_entry: &Entry,
        spice_tls_check: &CheckButton,
        spice_ca_cert_entry: &Entry,
        spice_skip_verify_check: &CheckButton,
        spice_usb_check: &CheckButton,
        spice_clipboard_check: &CheckButton,
        spice_compression_dropdown: &DropDown,
        zt_provider_dropdown: &DropDown,
        zt_aws_target: &Entry,
        zt_aws_profile: &Entry,
        zt_aws_region: &Entry,
        zt_gcp_instance: &Entry,
        zt_gcp_zone: &Entry,
        zt_gcp_project: &Entry,
        zt_azure_bastion_resource_id: &Entry,
        zt_azure_bastion_rg: &Entry,
        zt_azure_bastion_name: &Entry,
        zt_azure_ssh_vm: &Entry,
        zt_azure_ssh_rg: &Entry,
        zt_oci_bastion_id: &Entry,
        zt_oci_target_id: &Entry,
        zt_oci_target_ip: &Entry,
        zt_cf_hostname: &Entry,
        zt_teleport_host: &Entry,
        zt_teleport_cluster: &Entry,
        zt_tailscale_host: &Entry,
        zt_boundary_target: &Entry,
        zt_boundary_addr: &Entry,
        zt_generic_command: &Entry,
        zt_custom_args: &Entry,
    ) -> ProtocolConfig {
        match protocol_idx {
            1 => Self::build_rdp_config(
                rdp_client_mode_dropdown,
                rdp_width_spin,
                rdp_height_spin,
                rdp_color_dropdown,
                rdp_audio_check,
                rdp_gateway_entry,
                rdp_custom_args_entry,
            ),
            2 => Self::build_vnc_config(
                vnc_client_mode_dropdown,
                vnc_encoding_entry,
                vnc_compression_spin,
                vnc_quality_spin,
                vnc_view_only_check,
                vnc_scaling_check,
                vnc_clipboard_check,
                vnc_custom_args_entry,
            ),
            3 => Self::build_spice_config(
                spice_tls_check,
                spice_ca_cert_entry,
                spice_skip_verify_check,
                spice_usb_check,
                spice_clipboard_check,
                spice_compression_dropdown,
            ),
            4 => Self::build_zerotrust_config(
                zt_provider_dropdown,
                zt_aws_target,
                zt_aws_profile,
                zt_aws_region,
                zt_gcp_instance,
                zt_gcp_zone,
                zt_gcp_project,
                zt_azure_bastion_resource_id,
                zt_azure_bastion_rg,
                zt_azure_bastion_name,
                zt_azure_ssh_vm,
                zt_azure_ssh_rg,
                zt_oci_bastion_id,
                zt_oci_target_id,
                zt_oci_target_ip,
                zt_cf_hostname,
                zt_teleport_host,
                zt_teleport_cluster,
                zt_tailscale_host,
                zt_boundary_target,
                zt_boundary_addr,
                zt_generic_command,
                zt_custom_args,
            ),
            _ => Self::build_ssh_config(
                ssh_auth_dropdown,
                ssh_key_source_dropdown,
                ssh_key_entry,
                ssh_proxy_entry,
                ssh_identities_only,
                ssh_control_master,
                ssh_agent_forwarding,
                ssh_startup_entry,
                ssh_options_entry,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_ssh_config(
        auth_dropdown: &DropDown,
        key_source_dropdown: &DropDown,
        key_entry: &Entry,
        proxy_entry: &Entry,
        identities_only: &CheckButton,
        control_master: &CheckButton,
        agent_forwarding: &CheckButton,
        startup_entry: &Entry,
        options_entry: &Entry,
    ) -> ProtocolConfig {
        let auth_method = match auth_dropdown.selected() {
            1 => SshAuthMethod::PublicKey,
            2 => SshAuthMethod::KeyboardInteractive,
            3 => SshAuthMethod::Agent,
            _ => SshAuthMethod::Password,
        };

        let key_path_text = key_entry.text();
        let key_source = match key_source_dropdown.selected() {
            1 => SshKeySource::File {
                path: std::path::PathBuf::from(key_path_text.as_str()),
            },
            2 => SshKeySource::Agent {
                fingerprint: String::new(),
                comment: String::new(),
            },
            _ => SshKeySource::Default,
        };

        let proxy_jump = proxy_entry.text();
        let startup_command = startup_entry.text();
        let custom_options_text = options_entry.text();

        let mut config = SshConfig {
            auth_method,
            key_source,
            key_path: None,
            agent_key_fingerprint: None,
            proxy_jump: if proxy_jump.is_empty() {
                None
            } else {
                Some(proxy_jump.into())
            },
            identities_only: identities_only.is_active(),
            use_control_master: control_master.is_active(),
            agent_forwarding: agent_forwarding.is_active(),
            startup_command: if startup_command.is_empty() {
                None
            } else {
                Some(startup_command.into())
            },
            custom_options: std::collections::HashMap::new(),
        };

        if !custom_options_text.is_empty() {
            for pair in custom_options_text.split(',') {
                if let Some((k, v)) = pair.split_once('=') {
                    config
                        .custom_options
                        .insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }

        ProtocolConfig::Ssh(config)
    }

    fn build_rdp_config(
        client_mode_dropdown: &DropDown,
        width_spin: &SpinButton,
        height_spin: &SpinButton,
        color_dropdown: &DropDown,
        audio_check: &CheckButton,
        gateway_entry: &Entry,
        custom_args_entry: &Entry,
    ) -> ProtocolConfig {
        let client_mode = if client_mode_dropdown.selected() == 1 {
            RdpClientMode::External
        } else {
            RdpClientMode::Embedded
        };

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let resolution = Resolution {
            width: width_spin.value() as u32,
            height: height_spin.value() as u32,
        };

        let color_depth: u8 = match color_dropdown.selected() {
            1 => 24,
            2 => 16,
            3 => 15,
            4 => 8,
            _ => 32,
        };

        let gateway_text = gateway_entry.text();
        let custom_args_text = custom_args_entry.text();

        let custom_args: Vec<String> = if custom_args_text.is_empty() {
            Vec::new()
        } else {
            custom_args_text
                .split_whitespace()
                .map(String::from)
                .collect()
        };

        ProtocolConfig::Rdp(RdpConfig {
            client_mode,
            resolution: Some(resolution),
            color_depth: Some(color_depth),
            audio_redirect: audio_check.is_active(),
            gateway: if gateway_text.is_empty() {
                None
            } else {
                Some(rustconn_core::models::RdpGateway {
                    hostname: gateway_text.to_string(),
                    port: 443,
                    username: None,
                })
            },
            shared_folders: Vec::new(),
            custom_args,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn build_vnc_config(
        client_mode_dropdown: &DropDown,
        encoding_entry: &Entry,
        compression_spin: &SpinButton,
        quality_spin: &SpinButton,
        view_only_check: &CheckButton,
        scaling_check: &CheckButton,
        clipboard_check: &CheckButton,
        custom_args_entry: &Entry,
    ) -> ProtocolConfig {
        let client_mode = if client_mode_dropdown.selected() == 1 {
            VncClientMode::External
        } else {
            VncClientMode::Embedded
        };

        let encoding = encoding_entry.text();
        let custom_args_text = custom_args_entry.text();

        let custom_args: Vec<String> = if custom_args_text.is_empty() {
            Vec::new()
        } else {
            custom_args_text
                .split_whitespace()
                .map(String::from)
                .collect()
        };

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        ProtocolConfig::Vnc(VncConfig {
            client_mode,
            encoding: if encoding.is_empty() {
                None
            } else {
                Some(encoding.into())
            },
            compression: Some(compression_spin.value() as u8),
            quality: Some(quality_spin.value() as u8),
            view_only: view_only_check.is_active(),
            scaling: scaling_check.is_active(),
            clipboard_enabled: clipboard_check.is_active(),
            custom_args,
        })
    }

    fn build_spice_config(
        tls_check: &CheckButton,
        ca_cert_entry: &Entry,
        skip_verify_check: &CheckButton,
        usb_check: &CheckButton,
        clipboard_check: &CheckButton,
        compression_dropdown: &DropDown,
    ) -> ProtocolConfig {
        let ca_cert = ca_cert_entry.text();
        let compression = match compression_dropdown.selected() {
            1 => Some(SpiceImageCompression::Off),
            2 => Some(SpiceImageCompression::Glz),
            3 => Some(SpiceImageCompression::Lz),
            4 => Some(SpiceImageCompression::Quic),
            _ => Some(SpiceImageCompression::Auto),
        };

        ProtocolConfig::Spice(SpiceConfig {
            tls_enabled: tls_check.is_active(),
            ca_cert_path: if ca_cert.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(ca_cert.as_str()))
            },
            skip_cert_verify: skip_verify_check.is_active(),
            usb_redirection: usb_check.is_active(),
            shared_folders: Vec::new(),
            clipboard_enabled: clipboard_check.is_active(),
            image_compression: compression,
        })
    }

    #[allow(clippy::too_many_arguments, clippy::similar_names)]
    fn build_zerotrust_config(
        provider_dropdown: &DropDown,
        aws_target: &Entry,
        aws_profile: &Entry,
        aws_region: &Entry,
        gcp_instance: &Entry,
        gcp_zone: &Entry,
        gcp_project: &Entry,
        azure_bastion_resource_id: &Entry,
        azure_bastion_rg: &Entry,
        azure_bastion_name: &Entry,
        azure_ssh_vm: &Entry,
        azure_ssh_rg: &Entry,
        oci_bastion_id: &Entry,
        oci_target_id: &Entry,
        oci_target_ip: &Entry,
        cf_hostname: &Entry,
        teleport_host: &Entry,
        teleport_cluster: &Entry,
        tailscale_host: &Entry,
        boundary_target: &Entry,
        boundary_addr: &Entry,
        generic_command: &Entry,
        custom_args: &Entry,
    ) -> ProtocolConfig {
        let custom_args_text = custom_args.text();
        let custom_args_vec: Vec<String> = if custom_args_text.is_empty() {
            Vec::new()
        } else {
            custom_args_text
                .split_whitespace()
                .map(String::from)
                .collect()
        };

        let provider_config = match provider_dropdown.selected() {
            0 => ZeroTrustProviderConfig::AwsSsm(AwsSsmConfig {
                target: aws_target.text().to_string(),
                profile: aws_profile.text().to_string(),
                region: if aws_region.text().is_empty() {
                    None
                } else {
                    Some(aws_region.text().to_string())
                },
            }),
            1 => ZeroTrustProviderConfig::GcpIap(GcpIapConfig {
                instance: gcp_instance.text().to_string(),
                zone: gcp_zone.text().to_string(),
                project: if gcp_project.text().is_empty() {
                    None
                } else {
                    Some(gcp_project.text().to_string())
                },
            }),
            2 => ZeroTrustProviderConfig::AzureBastion(AzureBastionConfig {
                target_resource_id: azure_bastion_resource_id.text().to_string(),
                resource_group: azure_bastion_rg.text().to_string(),
                bastion_name: azure_bastion_name.text().to_string(),
            }),
            3 => ZeroTrustProviderConfig::AzureSsh(AzureSshConfig {
                vm_name: azure_ssh_vm.text().to_string(),
                resource_group: azure_ssh_rg.text().to_string(),
            }),
            4 => ZeroTrustProviderConfig::OciBastion(OciBastionConfig {
                bastion_id: oci_bastion_id.text().to_string(),
                target_resource_id: oci_target_id.text().to_string(),
                target_private_ip: oci_target_ip.text().to_string(),
                ssh_public_key_file: std::path::PathBuf::new(),
                session_ttl: 1800,
            }),
            5 => ZeroTrustProviderConfig::CloudflareAccess(CloudflareAccessConfig {
                hostname: cf_hostname.text().to_string(),
                username: None,
            }),
            6 => ZeroTrustProviderConfig::Teleport(TeleportConfig {
                host: teleport_host.text().to_string(),
                username: None,
                cluster: if teleport_cluster.text().is_empty() {
                    None
                } else {
                    Some(teleport_cluster.text().to_string())
                },
            }),
            7 => ZeroTrustProviderConfig::TailscaleSsh(TailscaleSshConfig {
                host: tailscale_host.text().to_string(),
                username: None,
            }),
            8 => ZeroTrustProviderConfig::Boundary(BoundaryConfig {
                target: boundary_target.text().to_string(),
                addr: if boundary_addr.text().is_empty() {
                    None
                } else {
                    Some(boundary_addr.text().to_string())
                },
            }),
            _ => ZeroTrustProviderConfig::Generic(GenericZeroTrustConfig {
                command_template: generic_command.text().to_string(),
            }),
        };

        let provider = match provider_dropdown.selected() {
            0 => ZeroTrustProvider::AwsSsm,
            1 => ZeroTrustProvider::GcpIap,
            2 => ZeroTrustProvider::AzureBastion,
            3 => ZeroTrustProvider::AzureSsh,
            4 => ZeroTrustProvider::OciBastion,
            5 => ZeroTrustProvider::CloudflareAccess,
            6 => ZeroTrustProvider::Teleport,
            7 => ZeroTrustProvider::TailscaleSsh,
            8 => ZeroTrustProvider::Boundary,
            _ => ZeroTrustProvider::Generic,
        };

        ProtocolConfig::ZeroTrust(ZeroTrustConfig {
            provider,
            provider_config,
            custom_args: custom_args_vec,
            detected_provider: None,
        })
    }

    /// Populates the dialog with an existing template for editing
    pub fn set_template(&self, template: &ConnectionTemplate) {
        self.window.set_title(Some("Edit Template"));
        self.save_button.set_label("Save");
        *self.editing_id.borrow_mut() = Some(template.id);

        self.name_entry.set_text(&template.name);
        if let Some(ref desc) = template.description {
            self.description_entry.set_text(desc);
        }

        let protocol_idx: u32 = match template.protocol {
            ProtocolType::Ssh => 0,
            ProtocolType::Rdp => 1,
            ProtocolType::Vnc => 2,
            ProtocolType::Spice => 3,
            ProtocolType::ZeroTrust => 4,
        };
        self.protocol_dropdown.set_selected(protocol_idx);
        self.protocol_stack
            .set_visible_child_name(match protocol_idx {
                1 => "rdp",
                2 => "vnc",
                3 => "spice",
                4 => "zerotrust",
                _ => "ssh",
            });

        self.host_entry.set_text(&template.host);
        self.port_spin.set_value(f64::from(template.port));

        if let Some(ref username) = template.username {
            self.username_entry.set_text(username);
        }

        self.tags_entry.set_text(&template.tags.join(", "));

        // Load protocol-specific config
        self.load_protocol_config(&template.protocol_config);
    }

    fn load_protocol_config(&self, config: &ProtocolConfig) {
        match config {
            ProtocolConfig::Ssh(ssh) => self.load_ssh_config(ssh),
            ProtocolConfig::Rdp(rdp) => self.load_rdp_config(rdp),
            ProtocolConfig::Vnc(vnc) => self.load_vnc_config(vnc),
            ProtocolConfig::Spice(spice) => self.load_spice_config(spice),
            ProtocolConfig::ZeroTrust(zt) => self.load_zerotrust_config(zt),
        }
    }

    fn load_ssh_config(&self, config: &SshConfig) {
        let auth_idx = match config.auth_method {
            SshAuthMethod::Password => 0,
            SshAuthMethod::PublicKey => 1,
            SshAuthMethod::KeyboardInteractive => 2,
            SshAuthMethod::Agent => 3,
        };
        self.ssh_auth_dropdown.set_selected(auth_idx);

        let key_source_idx = match &config.key_source {
            SshKeySource::Default => 0,
            SshKeySource::File { .. } => 1,
            SshKeySource::Agent { .. } => 2,
        };
        self.ssh_key_source_dropdown.set_selected(key_source_idx);

        if let SshKeySource::File { path } = &config.key_source {
            self.ssh_key_entry.set_text(&path.display().to_string());
        }
        if let Some(ref proxy) = config.proxy_jump {
            self.ssh_proxy_entry.set_text(proxy);
        }
        self.ssh_identities_only.set_active(config.identities_only);
        self.ssh_control_master
            .set_active(config.use_control_master);
        self.ssh_agent_forwarding
            .set_active(config.agent_forwarding);
        if let Some(ref cmd) = config.startup_command {
            self.ssh_startup_entry.set_text(cmd);
        }
        if !config.custom_options.is_empty() {
            let opts: Vec<String> = config
                .custom_options
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            self.ssh_options_entry.set_text(&opts.join(", "));
        }
    }

    fn load_rdp_config(&self, config: &RdpConfig) {
        let mode_idx = match config.client_mode {
            RdpClientMode::Embedded => 0,
            RdpClientMode::External => 1,
        };
        self.rdp_client_mode_dropdown.set_selected(mode_idx);
        if let Some(ref res) = config.resolution {
            self.rdp_width_spin.set_value(f64::from(res.width));
            self.rdp_height_spin.set_value(f64::from(res.height));
        }
        let color_idx = match config.color_depth {
            Some(24) => 1,
            Some(16) => 2,
            Some(15) => 3,
            Some(8) => 4,
            _ => 0,
        };
        self.rdp_color_dropdown.set_selected(color_idx);
        self.rdp_audio_check.set_active(config.audio_redirect);
        if let Some(ref gw) = config.gateway {
            self.rdp_gateway_entry.set_text(&gw.hostname);
        }
        if !config.custom_args.is_empty() {
            self.rdp_custom_args_entry
                .set_text(&config.custom_args.join(" "));
        }
    }

    fn load_vnc_config(&self, config: &VncConfig) {
        let mode_idx = match config.client_mode {
            VncClientMode::Embedded => 0,
            VncClientMode::External => 1,
        };
        self.vnc_client_mode_dropdown.set_selected(mode_idx);
        if let Some(ref enc) = config.encoding {
            self.vnc_encoding_entry.set_text(enc);
        }
        if let Some(c) = config.compression {
            self.vnc_compression_spin.set_value(f64::from(c));
        }
        if let Some(q) = config.quality {
            self.vnc_quality_spin.set_value(f64::from(q));
        }
        self.vnc_view_only_check.set_active(config.view_only);
        self.vnc_scaling_check.set_active(config.scaling);
        self.vnc_clipboard_check
            .set_active(config.clipboard_enabled);
        if !config.custom_args.is_empty() {
            self.vnc_custom_args_entry
                .set_text(&config.custom_args.join(" "));
        }
    }

    fn load_spice_config(&self, config: &SpiceConfig) {
        self.spice_tls_check.set_active(config.tls_enabled);
        if let Some(ref cert) = config.ca_cert_path {
            self.spice_ca_cert_entry
                .set_text(&cert.display().to_string());
        }
        self.spice_skip_verify_check
            .set_active(config.skip_cert_verify);
        self.spice_usb_check.set_active(config.usb_redirection);
        self.spice_clipboard_check
            .set_active(config.clipboard_enabled);
        let comp_idx = match config.image_compression {
            Some(SpiceImageCompression::Auto) | None => 0,
            Some(SpiceImageCompression::Off) => 1,
            Some(SpiceImageCompression::Glz) => 2,
            Some(SpiceImageCompression::Lz) => 3,
            Some(SpiceImageCompression::Quic) => 4,
        };
        self.spice_compression_dropdown.set_selected(comp_idx);
    }

    fn load_zerotrust_config(&self, config: &ZeroTrustConfig) {
        let provider_idx = match config.provider {
            ZeroTrustProvider::AwsSsm => 0,
            ZeroTrustProvider::GcpIap => 1,
            ZeroTrustProvider::AzureBastion => 2,
            ZeroTrustProvider::AzureSsh => 3,
            ZeroTrustProvider::OciBastion => 4,
            ZeroTrustProvider::CloudflareAccess => 5,
            ZeroTrustProvider::Teleport => 6,
            ZeroTrustProvider::TailscaleSsh => 7,
            ZeroTrustProvider::Boundary => 8,
            ZeroTrustProvider::Generic => 9,
        };
        self.zt_provider_dropdown.set_selected(provider_idx);

        let stack_name = match config.provider {
            ZeroTrustProvider::AwsSsm => "aws_ssm",
            ZeroTrustProvider::GcpIap => "gcp_iap",
            ZeroTrustProvider::AzureBastion => "azure_bastion",
            ZeroTrustProvider::AzureSsh => "azure_ssh",
            ZeroTrustProvider::OciBastion => "oci_bastion",
            ZeroTrustProvider::CloudflareAccess => "cloudflare",
            ZeroTrustProvider::Teleport => "teleport",
            ZeroTrustProvider::TailscaleSsh => "tailscale",
            ZeroTrustProvider::Boundary => "boundary",
            ZeroTrustProvider::Generic => "generic",
        };
        self.zt_provider_stack.set_visible_child_name(stack_name);

        match &config.provider_config {
            ZeroTrustProviderConfig::AwsSsm(c) => {
                self.zt_aws_target_entry.set_text(&c.target);
                self.zt_aws_profile_entry.set_text(&c.profile);
                if let Some(ref r) = c.region {
                    self.zt_aws_region_entry.set_text(r);
                }
            }
            ZeroTrustProviderConfig::GcpIap(c) => {
                self.zt_gcp_instance_entry.set_text(&c.instance);
                self.zt_gcp_zone_entry.set_text(&c.zone);
                if let Some(ref p) = c.project {
                    self.zt_gcp_project_entry.set_text(p);
                }
            }
            ZeroTrustProviderConfig::AzureBastion(c) => {
                self.zt_azure_bastion_resource_id_entry
                    .set_text(&c.target_resource_id);
                self.zt_azure_bastion_rg_entry.set_text(&c.resource_group);
                self.zt_azure_bastion_name_entry.set_text(&c.bastion_name);
            }
            ZeroTrustProviderConfig::AzureSsh(c) => {
                self.zt_azure_ssh_vm_entry.set_text(&c.vm_name);
                self.zt_azure_ssh_rg_entry.set_text(&c.resource_group);
            }
            ZeroTrustProviderConfig::OciBastion(c) => {
                self.zt_oci_bastion_id_entry.set_text(&c.bastion_id);
                self.zt_oci_target_id_entry.set_text(&c.target_resource_id);
                self.zt_oci_target_ip_entry.set_text(&c.target_private_ip);
            }
            ZeroTrustProviderConfig::CloudflareAccess(c) => {
                self.zt_cf_hostname_entry.set_text(&c.hostname);
            }
            ZeroTrustProviderConfig::Teleport(c) => {
                self.zt_teleport_host_entry.set_text(&c.host);
                if let Some(ref cl) = c.cluster {
                    self.zt_teleport_cluster_entry.set_text(cl);
                }
            }
            ZeroTrustProviderConfig::TailscaleSsh(c) => {
                self.zt_tailscale_host_entry.set_text(&c.host);
            }
            ZeroTrustProviderConfig::Boundary(c) => {
                self.zt_boundary_target_entry.set_text(&c.target);
                if let Some(ref a) = c.addr {
                    self.zt_boundary_addr_entry.set_text(a);
                }
            }
            ZeroTrustProviderConfig::Generic(c) => {
                self.zt_generic_command_entry.set_text(&c.command_template);
            }
        }

        if !config.custom_args.is_empty() {
            self.zt_custom_args_entry
                .set_text(&config.custom_args.join(" "));
        }
    }

    /// Runs the dialog and calls the callback with the result
    pub fn run<F: Fn(Option<ConnectionTemplate>) + 'static>(&self, cb: F) {
        *self.on_save.borrow_mut() = Some(Box::new(cb));
        self.window.present();
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub const fn window(&self) -> &adw::Window {
        &self.window
    }
}

/// Template manager dialog for listing and managing templates
pub struct TemplateManagerDialog {
    window: adw::Window,
    templates_list: ListBox,
    state_templates: Rc<RefCell<Vec<ConnectionTemplate>>>,
    on_template_selected: Rc<RefCell<Option<Box<dyn Fn(Option<ConnectionTemplate>)>>>>,
    on_new: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_edit: Rc<RefCell<Option<Box<dyn Fn(ConnectionTemplate)>>>>,
    on_delete: Rc<RefCell<Option<Box<dyn Fn(Uuid)>>>>,
}

impl TemplateManagerDialog {
    /// Creates a new template manager dialog
    #[must_use]
    pub fn new(parent: Option<&gtk4::Window>) -> Self {
        let window = adw::Window::builder()
            .title("Manage Templates")
            .modal(true)
            .default_width(750)
            .default_height(500)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
        }

        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);
        let close_btn = Button::builder().label("Close").build();
        let create_conn_btn = Button::builder()
            .label("Create")
            .css_classes(["suggested-action"])
            .sensitive(false)
            .build();
        header.pack_start(&close_btn);
        header.pack_end(&create_conn_btn);

        // Close button handler
        let window_clone = window.clone();
        close_btn.connect_clicked(move |_| {
            window_clone.close();
        });

        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let filter_box = GtkBox::new(Orientation::Horizontal, 8);
        let filter_label = Label::new(Some("Filter by protocol:"));
        let protocols = StringList::new(&["All", "SSH", "RDP", "VNC", "SPICE"]);
        let filter_dropdown = DropDown::builder().model(&protocols).build();
        filter_box.append(&filter_label);
        filter_box.append(&filter_dropdown);
        content.append(&filter_box);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let templates_list = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .css_classes(["boxed-list"])
            .build();
        scrolled.set_child(Some(&templates_list));
        content.append(&scrolled);

        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);

        let edit_btn = Button::builder().label("Edit").sensitive(false).build();
        let delete_btn = Button::builder().label("Delete").sensitive(false).build();
        let new_template_btn = Button::builder()
            .label("Create Template")
            .sensitive(true)
            .css_classes(["suggested-action"])
            .build();

        button_box.append(&edit_btn);
        button_box.append(&delete_btn);
        button_box.append(&new_template_btn);
        content.append(&button_box);

        // Use ToolbarView for adw::Window
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);
        main_box.append(&content);
        window.set_content(Some(&main_box));

        let state_templates: Rc<RefCell<Vec<ConnectionTemplate>>> =
            Rc::new(RefCell::new(Vec::new()));
        let on_template_selected: Rc<RefCell<Option<Box<dyn Fn(Option<ConnectionTemplate>)>>>> =
            Rc::new(RefCell::new(None));
        let on_new: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        let on_edit: Rc<RefCell<Option<Box<dyn Fn(ConnectionTemplate)>>>> =
            Rc::new(RefCell::new(None));
        let on_delete: Rc<RefCell<Option<Box<dyn Fn(Uuid)>>>> = Rc::new(RefCell::new(None));

        let edit_clone = edit_btn.clone();
        let delete_clone = delete_btn.clone();
        let create_conn_clone = create_conn_btn.clone();
        templates_list.connect_row_selected(move |_, row| {
            let has_selection = row.is_some();
            edit_clone.set_sensitive(has_selection);
            delete_clone.set_sensitive(has_selection);
            create_conn_clone.set_sensitive(has_selection);
        });

        // "Create Template" button - creates a new template
        let on_new_clone = on_new.clone();
        new_template_btn.connect_clicked(move |_| {
            if let Some(ref cb) = *on_new_clone.borrow() {
                cb();
            }
        });

        let on_edit_clone = on_edit.clone();
        let state_templates_edit = state_templates.clone();
        let templates_list_edit = templates_list.clone();
        edit_btn.connect_clicked(move |_| {
            if let Some(row) = templates_list_edit.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("template-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        let templates = state_templates_edit.borrow();
                        if let Some(template) = templates.iter().find(|t| t.id == id) {
                            if let Some(ref cb) = *on_edit_clone.borrow() {
                                cb(template.clone());
                            }
                        }
                    }
                }
            }
        });

        let on_delete_clone = on_delete.clone();
        let templates_list_delete = templates_list.clone();
        delete_btn.connect_clicked(move |_| {
            if let Some(row) = templates_list_delete.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("template-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        if let Some(ref cb) = *on_delete_clone.borrow() {
                            cb(id);
                        }
                    }
                }
            }
        });

        // "Create Connection" button in header - creates connection from selected template
        let on_selected_clone = on_template_selected.clone();
        let state_templates_use = state_templates.clone();
        let templates_list_use = templates_list.clone();
        let window_use = window.clone();
        create_conn_btn.connect_clicked(move |_| {
            if let Some(row) = templates_list_use.selected_row() {
                if let Some(id_str) = row.widget_name().as_str().strip_prefix("template-") {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        let templates = state_templates_use.borrow();
                        if let Some(template) = templates.iter().find(|t| t.id == id) {
                            if let Some(ref cb) = *on_selected_clone.borrow() {
                                cb(Some(template.clone()));
                            }
                            window_use.close();
                        }
                    }
                }
            }
        });

        // Double-click on template row - creates connection from template
        let gesture = gtk4::GestureClick::new();
        gesture.set_button(1); // Left mouse button
        let on_selected_dblclick = on_template_selected.clone();
        let state_templates_dblclick = state_templates.clone();
        let templates_list_dblclick = templates_list.clone();
        let window_dblclick = window.clone();
        gesture.connect_pressed(move |gesture, n_press, _x, y| {
            if n_press == 2 {
                // Double-click
                if let Some(row) = templates_list_dblclick.row_at_y(y as i32) {
                    if let Some(id_str) = row.widget_name().as_str().strip_prefix("template-") {
                        if let Ok(id) = Uuid::parse_str(id_str) {
                            let templates = state_templates_dblclick.borrow();
                            if let Some(template) = templates.iter().find(|t| t.id == id) {
                                if let Some(ref cb) = *on_selected_dblclick.borrow() {
                                    cb(Some(template.clone()));
                                }
                                window_dblclick.close();
                            }
                        }
                    }
                }
                gesture.set_state(gtk4::EventSequenceState::Claimed);
            }
        });
        templates_list.add_controller(gesture);

        Self {
            window,
            templates_list,
            state_templates,
            on_template_selected,
            on_new,
            on_edit,
            on_delete,
        }
    }

    /// Sets the templates to display
    pub fn set_templates(&self, templates: Vec<ConnectionTemplate>) {
        *self.state_templates.borrow_mut() = templates;
        self.refresh_list(None);
    }

    /// Refreshes the templates list with optional protocol filter
    pub fn refresh_list(&self, protocol_filter: Option<ProtocolType>) {
        while let Some(row) = self.templates_list.row_at_index(0) {
            self.templates_list.remove(&row);
        }

        let templates = self.state_templates.borrow();

        let mut ssh_templates: Vec<&ConnectionTemplate> = Vec::new();
        let mut rdp_templates: Vec<&ConnectionTemplate> = Vec::new();
        let mut vnc_templates: Vec<&ConnectionTemplate> = Vec::new();
        let mut spice_templates: Vec<&ConnectionTemplate> = Vec::new();

        for template in templates.iter() {
            if let Some(filter) = protocol_filter {
                if template.protocol != filter {
                    continue;
                }
            }
            match template.protocol {
                ProtocolType::Ssh | ProtocolType::ZeroTrust => ssh_templates.push(template),
                ProtocolType::Rdp => rdp_templates.push(template),
                ProtocolType::Vnc => vnc_templates.push(template),
                ProtocolType::Spice => spice_templates.push(template),
            }
        }

        if !ssh_templates.is_empty() && protocol_filter.is_none() {
            self.add_section_header("SSH Templates");
        }
        for template in ssh_templates {
            self.add_template_row(template);
        }

        if !rdp_templates.is_empty() && protocol_filter.is_none() {
            self.add_section_header("RDP Templates");
        }
        for template in rdp_templates {
            self.add_template_row(template);
        }

        if !vnc_templates.is_empty() && protocol_filter.is_none() {
            self.add_section_header("VNC Templates");
        }
        for template in vnc_templates {
            self.add_template_row(template);
        }

        if !spice_templates.is_empty() && protocol_filter.is_none() {
            self.add_section_header("SPICE Templates");
        }
        for template in spice_templates {
            self.add_template_row(template);
        }
    }

    fn add_section_header(&self, title: &str) {
        let label = Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .margin_top(8)
            .margin_bottom(4)
            .margin_start(8)
            .build();
        let row = ListBoxRow::builder()
            .child(&label)
            .selectable(false)
            .activatable(false)
            .build();
        self.templates_list.append(&row);
    }

    fn add_template_row(&self, template: &ConnectionTemplate) {
        let hbox = GtkBox::new(Orientation::Horizontal, 8);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);

        let icon_name = match template.protocol {
            ProtocolType::Ssh => "utilities-terminal-symbolic",
            ProtocolType::Rdp => "computer-symbolic",
            ProtocolType::Vnc => "video-display-symbolic",
            ProtocolType::Spice => "video-display-symbolic",
            ProtocolType::ZeroTrust => "cloud-symbolic",
        };
        let icon = gtk4::Image::from_icon_name(icon_name);
        hbox.append(&icon);

        let info_box = GtkBox::new(Orientation::Vertical, 2);
        info_box.set_hexpand(true);

        let name_label = Label::builder()
            .label(&template.name)
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        info_box.append(&name_label);

        let details = if let Some(ref desc) = template.description {
            desc.clone()
        } else {
            let mut parts = Vec::new();
            if !template.host.is_empty() {
                parts.push(format!("Host: {}", template.host));
            }
            parts.push(format!("Port: {}", template.port));
            if let Some(ref user) = template.username {
                parts.push(format!("User: {user}"));
            }
            parts.join(" | ")
        };

        let details_label = Label::builder()
            .label(&details)
            .halign(gtk4::Align::Start)
            .css_classes(["dim-label"])
            .build();
        info_box.append(&details_label);

        hbox.append(&info_box);

        let row = ListBoxRow::builder().child(&hbox).build();
        row.set_widget_name(&format!("template-{}", template.id));
        self.templates_list.append(&row);
    }

    /// Gets the currently selected template
    #[must_use]
    pub fn get_selected_template(&self) -> Option<ConnectionTemplate> {
        if let Some(row) = self.templates_list.selected_row() {
            if let Some(id_str) = row.widget_name().as_str().strip_prefix("template-") {
                if let Ok(id) = Uuid::parse_str(id_str) {
                    let templates = self.state_templates.borrow();
                    return templates.iter().find(|t| t.id == id).cloned();
                }
            }
        }
        None
    }

    /// Returns a reference to the underlying window
    #[must_use]
    pub const fn window(&self) -> &adw::Window {
        &self.window
    }

    /// Returns a reference to the templates list
    #[must_use]
    pub const fn templates_list(&self) -> &ListBox {
        &self.templates_list
    }

    /// Returns a reference to the state templates
    #[must_use]
    pub fn state_templates(&self) -> &Rc<RefCell<Vec<ConnectionTemplate>>> {
        &self.state_templates
    }

    /// Presents the dialog
    pub fn present(&self) {
        self.window.present();
    }

    /// Sets the callback for creating a new template
    pub fn set_on_new<F: Fn() + 'static>(&self, cb: F) {
        *self.on_new.borrow_mut() = Some(Box::new(cb));
    }

    /// Sets the callback for editing a template
    pub fn set_on_edit<F: Fn(ConnectionTemplate) + 'static>(&self, cb: F) {
        *self.on_edit.borrow_mut() = Some(Box::new(cb));
    }

    /// Sets the callback for deleting a template
    pub fn set_on_delete<F: Fn(Uuid) + 'static>(&self, cb: F) {
        *self.on_delete.borrow_mut() = Some(Box::new(cb));
    }

    /// Sets the callback for selecting a template to use
    pub fn set_on_template_selected<F: Fn(Option<ConnectionTemplate>) + 'static>(&self, cb: F) {
        *self.on_template_selected.borrow_mut() = Some(Box::new(cb));
    }
}
