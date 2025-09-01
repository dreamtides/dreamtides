use ability_data::ability::{Ability, DisplayedAbility};

use crate::card_definitions::base_card_definition_raw::BaseCardDefinitionRaw;

pub trait BaseCardDefinitionType {
    fn id_string(&self) -> String;
    fn rules_text_en_us(&self) -> &str;
    fn name_en_us(&self) -> &str;
    fn prompts_en_us(&self) -> Option<&str>;
    fn abilities_ref(&self) -> Option<&Vec<Ability>>;
    fn abilities_mut(&mut self) -> &mut Option<Vec<Ability>>;
    fn displayed_abilities_ref(&self) -> Option<&Vec<DisplayedAbility>>;
    fn displayed_abilities_mut(&mut self) -> &mut Option<Vec<DisplayedAbility>>;
    fn image_number(&self) -> &str;
    fn image_directory(&self) -> &'static str;
}

impl BaseCardDefinitionType for BaseCardDefinitionRaw {
    fn id_string(&self) -> String {
        format!("{}", self.id.0)
    }

    fn rules_text_en_us(&self) -> &str {
        self.rules_text_en_us.as_str()
    }

    fn name_en_us(&self) -> &str {
        self.name_en_us.as_str()
    }

    fn prompts_en_us(&self) -> Option<&str> {
        self.prompts_en_us.as_deref()
    }

    fn abilities_ref(&self) -> Option<&Vec<Ability>> {
        self.abilities.as_ref()
    }

    fn abilities_mut(&mut self) -> &mut Option<Vec<Ability>> {
        &mut self.abilities
    }

    fn displayed_abilities_ref(&self) -> Option<&Vec<DisplayedAbility>> {
        self.displayed_abilities.as_ref()
    }

    fn displayed_abilities_mut(&mut self) -> &mut Option<Vec<DisplayedAbility>> {
        &mut self.displayed_abilities
    }

    fn image_number(&self) -> &str {
        self.image_number.as_str()
    }

    fn image_directory(&self) -> &'static str {
        "Standard"
    }
}

impl BaseCardDefinitionType for super::dreamwell_card_definition::DreamwellCardDefinitionRaw {
    fn id_string(&self) -> String {
        format!("{}", self.id.0)
    }

    fn rules_text_en_us(&self) -> &str {
        self.rules_text_en_us.as_str()
    }

    fn name_en_us(&self) -> &str {
        self.name_en_us.as_str()
    }

    fn prompts_en_us(&self) -> Option<&str> {
        self.prompts_en_us.as_deref()
    }

    fn abilities_ref(&self) -> Option<&Vec<Ability>> {
        self.abilities.as_ref()
    }

    fn abilities_mut(&mut self) -> &mut Option<Vec<Ability>> {
        &mut self.abilities
    }

    fn displayed_abilities_ref(&self) -> Option<&Vec<DisplayedAbility>> {
        self.displayed_abilities.as_ref()
    }

    fn displayed_abilities_mut(&mut self) -> &mut Option<Vec<DisplayedAbility>> {
        &mut self.displayed_abilities
    }

    fn image_number(&self) -> &str {
        self.image_number.as_str()
    }

    fn image_directory(&self) -> &'static str {
        "Dreamwell"
    }
}
