use minestom::{Component, component};

pub fn create_ban_message(reason: &str, seconds_left: Option<u64>) -> Component {
    let reason_component = component!("Reason: {}", reason)
        .red()
        .chain(component!("").white());

    let duration_component = match seconds_left {
        None => component!("Permanent Ban").red(),
        Some(sec) => {
            let days = sec / 86_400;
            let hours = (sec % 86_400) / 3_600;
            let minutes = (sec % 3_600) / 60;
            let seconds = sec % 60;

            component!(
                "Time Left: {} days, {} hours, {} minutes, {} seconds",
                days,
                hours,
                minutes,
                seconds
            )
            .red()
            .chain(component!("").white())
        }
    };

    component!("You are banned from this server!")
        .red()
        .chain_newline(reason_component)
        .chain_newline(duration_component)
}
