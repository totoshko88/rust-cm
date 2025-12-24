use ironrdp::pdu::rdp::client_info::TimezoneInfo;
use ironrdp::pdu::rdp::capability_sets::CodecProperty;

fn main() {
    let tz = TimezoneInfo {
        bias: 0,
        standard_name: [0; 32],
        standard_date: unsafe { std::mem::zeroed() },
        standard_bias: 0,
        daylight_name: [0; 32],
        daylight_date: unsafe { std::mem::zeroed() },
        daylight_bias: 0,
    };

    let _ = CodecProperty::ImageRemoteFx;
}
