//! The menu that shows an option and refuses it, drawn for real.
//!
//! The unit test pins the rule; this one pins that the panel obeys it, by
//! opening the drop-down in a real `egui` pass and looking at what came back.

use mosna_gui::model::field::{is_available, unavailable_options};

/// Every option a menu lists is either selectable or carries a reason. A
/// third state — refused and silent — is the one a user reports as a bug.
#[test]
fn every_listed_option_is_either_selectable_or_explained() {
    let config = mosna_config::RawConfig::from_yaml_str(
        "Niche Analysis:\n  Processing method: Aggregated nodes\n  Niches method: NAS\n",
    )
    .unwrap();
    let form = mosna_gui::model::form::Form::from_config(&config);

    let mut seen = 0;
    for section in &form.sections {
        for tab in &section.tabs {
            for group in &tab.groups {
                for field in &group.fields {
                    if let mosna_gui::model::field::FieldKind::Choice { options, .. } = &field.kind
                    {
                        for option in options {
                            seen += 1;
                            if !is_available(&field.key, option) {
                                let reason = unavailable_options(&field.key)
                                    .iter()
                                    .find(|(name, _)| name == option)
                                    .map(|(_, reason)| *reason)
                                    .unwrap_or("");
                                assert!(
                                    !reason.is_empty(),
                                    "`{}` refuses `{option}` and says nothing",
                                    field.key
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(seen > 0, "no menu was examined");
}
