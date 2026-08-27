use ratatui::Frame;

use super::render_form;

pub fn render_access_form(frame: &mut Frame, action: usize, values: &[String], field: usize) {
    let specs: (&str, Vec<(&str, &str)>) = match action {
        0 => (
            "New Department",
            vec![
                ("Name", "Operations"),
                ("Code", "OPS"),
                ("Description", "optional"),
            ],
        ),
        1 => (
            "New Designation",
            vec![
                ("Name", "Supervisor"),
                ("Code", "SUP"),
                ("Description", "optional"),
                ("Senior ID", "optional UUID"),
            ],
        ),
        2 => (
            "New RDD",
            vec![
                ("Role ID", "UUID"),
                ("Designation ID", "UUID"),
                ("Department ID", "UUID"),
            ],
        ),
        3 => (
            "Assign User Context",
            vec![
                ("User ID", "UUID"),
                ("RDD ID", "UUID"),
                ("Attributes JSON", "{}"),
            ],
        ),
        4 => (
            "New Permission Group",
            vec![("Name", "Front desk"), ("Description", "optional")],
        ),
        5 => (
            "New Permission Scope",
            vec![
                ("Permission ID", "UUID"),
                ("Name", "Assigned ward"),
                ("Resource type", "ward"),
                ("Resource ID", "optional UUID"),
                ("Conditions JSON", "{}"),
            ],
        ),
        6 => (
            "Add Group Permission",
            vec![("Group ID", "UUID"), ("Permission ID", "UUID")],
        ),
        7 => (
            "Link RDD Permission Group",
            vec![("RDD ID", "UUID"), ("Group ID", "UUID")],
        ),
        _ => (
            "Assign Permission Scope",
            vec![
                ("URDD ID", "UUID"),
                ("Permission ID", "UUID"),
                ("Scope ID", "UUID"),
            ],
        ),
    };
    let inputs = specs
        .1
        .iter()
        .zip(values.iter())
        .map(|((label, hint), value)| (*label, value.as_str(), *hint))
        .collect::<Vec<_>>();
    render_form(frame, specs.0, &inputs, field);
}

pub fn access_field_count(action: usize) -> usize {
    match action {
        0 | 2 | 3 | 8 => 3,
        1 => 4,
        4 | 6 | 7 => 2,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::access_field_count;

    #[test]
    fn every_access_action_has_the_expected_complete_form() {
        assert_eq!(
            (0..=8).map(access_field_count).collect::<Vec<_>>(),
            vec![3, 4, 3, 3, 2, 5, 2, 2, 3]
        );
    }
}
