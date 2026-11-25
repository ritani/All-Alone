//! The codex: found notes, photographs, radio fragments and the field guide.
//! The story of the Stillness is only ever told through these.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS))]
#[serde(rename_all = "snake_case")]
pub enum CodexKind {
    Note,
    Photograph,
    Radio,
    FieldGuide,
}

impl CodexKind {
    pub const ALL: [CodexKind; 4] = [
        CodexKind::Note,
        CodexKind::Photograph,
        CodexKind::Radio,
        CodexKind::FieldGuide,
    ];
    pub fn name(self) -> &'static str {
        match self {
            CodexKind::Note => "Notes",
            CodexKind::Photograph => "Photographs",
            CodexKind::Radio => "Radio",
            CodexKind::FieldGuide => "Field guide",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            CodexKind::Note => "journal",
            CodexKind::Photograph => "photo",
            CodexKind::Radio => "radio",
            CodexKind::FieldGuide => "stilled",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CodexDef {
    pub id: &'static str,
    pub kind: CodexKind,
    pub title: &'static str,
    pub text: &'static str,
    /// A place this entry tells you about.
    pub reveals: Option<&'static str>,
    /// Radio only: the earliest day it can be heard, and the radio level needed.
    pub day: u32,
    pub radio: u8,
}

const C: CodexDef = CodexDef {
    id: "",
    kind: CodexKind::Note,
    title: "",
    text: "",
    reveals: None,
    day: 0,
    radio: 0,
};

pub static CODEX: &[CodexDef] = &[
    // Notes
    CodexDef {
        id: "note_orme",
        title: "Note on the kitchen table",
        text: "Gone up to the town to see about the noise. Keep the sluices open, whatever happens. It is the water that drowns it. Back by morning. E. Orme.",
        ..C
    },
    CodexDef {
        id: "note_sluice",
        title: "Weir keeper's tally",
        text: "Gates eased two turns, river high. Lowe's men to come up from the boatyard Thursday about the dock gates, they leak worse than ever. Fish poor. Hum again tonight, could not hear it over the weir but felt it in the rail.",
        reveals: Some("boatyard"),
        ..C
    },
    CodexDef {
        id: "note_doctor_card",
        title: "Appointment card",
        text: "Dr. R. Hale, The Surgery, Quarry Lane. Tuesday, 10 o'clock. On the back, in pencil: ears again. worse at night. ask about the mill.",
        reveals: Some("surgery"),
        ..C
    },
    CodexDef {
        id: "note_school_letter",
        title: "Letter from the school",
        text: "Dear Mrs Kell, Arthur has been absent three days and was found again in the yard after dark, standing. He will not say why. He hums. Several of the children hum. Please call at the Board School. M. Penrose, Headmistress.",
        reveals: Some("school"),
        ..C
    },
    CodexDef {
        id: "note_ringer",
        title: "Bell-ringer's book",
        text: "Practice cancelled, Mr Ash unwell. The tenor is humming on its own again, low, a long while after I have stopped it with my hand. Vicar says damp in the frame. It is not damp.",
        ..C
    },
    CodexDef {
        id: "note_tower",
        title: "Pencil on the tower wall",
        text: "2.10. It rang once by itself and held. I can see the whole town from up here and none of it is moving. There is smoke at Hollin farm. I will wait for light.",
        reveals: Some("farm"),
        ..C
    },
    CodexDef {
        id: "note_shift_card",
        title: "Clocking card",
        text: "Night shift, engine house, J. Barrow. In 22.00. No time out. Every card in the rack is the same.",
        ..C
    },
    CodexDef {
        id: "note_mill_ledger",
        title: "Mill ledger",
        text: "New engine running in. Coal down, output up. Paraffin for the lamps to be fetched from the wharf store, padlock key with Lowe. Complaints from the town about noise at night: none from us, the engine is stopped at ten.",
        reveals: Some("wharf"),
        ..C
    },
    CodexDef {
        id: "note_engineer",
        title: "Engineer's log",
        text: "She runs sweet but there is a note in her I cannot tune out. Drew the fires at ten. At one the flywheel was turning, slow, no steam. The floor is warm. I think the note comes up through the floor and the engine only answers it.",
        ..C
    },
    CodexDef {
        id: "note_register",
        title: "School register",
        text: "A column of absences in red, growing down the page for three weeks. In the margin: they are not ill. they stand in the yard and face the mill. they say it is singing.",
        ..C
    },
    CodexDef {
        id: "note_hale_cases",
        title: "Case notes",
        text: "Fourteen patients this month with a ringing in the ears, worst near the river at the mill end, gone near the weir. Two found standing in the street at night, unresponsive, recovered by morning. Wrote to the county. No reply.",
        ..C
    },
    CodexDef {
        id: "note_hale_last",
        title: "Hale's last page",
        text: "It is louder than it has ever been. Margaret is at the window and will not come away. If anyone reads this: stop your ears. Wax, cloth, anything. Get near running water. It cannot reach you there.",
        ..C
    },
    CodexDef {
        id: "note_lowe",
        title: "Note pinned in the loft",
        text: "Wren's ferry: hull patch made up, on the trestle in the dock. Coil is at Pell's for rewinding. Belt to be cut from the mill's spare belting in the engine house. Fuel at the wharf. Tell Ada it will be ready by the frost.",
        reveals: Some("garage"),
        ..C
    },
    CodexDef {
        id: "note_pell",
        title: "Invoice, unpaid",
        text: "To rewinding one spark coil, ferry Kittiwake, T. Wren. Not done: windings split, needs fresh wire and a good bench. Left in the pit under the van to keep it from thieves. Fuel to be collected from the coal wharf by the owner.",
        reveals: Some("wharf"),
        ..C
    },
    CodexDef {
        id: "note_daybook",
        title: "Station day book",
        text: "01.50 Complaint, noise, Mill Street. 02.05 Complaint, noise, Quarry Lane. 02.09 Complaint, noise, everywhere. 02.10. The pen line runs off the page.",
        ..C
    },
    CodexDef {
        id: "note_wharfinger",
        title: "Wharfinger's ledger",
        text: "Last barge down to the estuary Friday. Skipper says the hum is in the water too, all the way to the sea, but weaker. He would not tie up overnight. Fuel drums in the store for Wren, paid.",
        ..C
    },
    CodexDef {
        id: "note_hollin",
        title: "Letter on the dresser",
        text: "We are going up to the tops with the sheep where it is quiet. Leave the dog, he will not come away from the house. Food in the cellar for whoever needs it. God keep you.",
        ..C
    },
    CodexDef {
        id: "note_ivy",
        title: "Note left by the stove",
        text: "Thank you for the warmth. I walked down from the county hospital along the water, the whole way. Keep to the water. They do not like it loud. Ivy.",
        ..C
    },
    CodexDef {
        id: "note_stranger",
        title: "Stranger's map",
        text: "A pencil map of the river with the weir, the wharf and the estuary marked. Beside the wharf: fuel, locked, key at Lowe's. Beside the estuary: boats. Beside the town: a ring, and the word no.",
        reveals: Some("wharf"),
        ..C
    },
    // Photographs
    CodexDef {
        id: "photo_wedding",
        kind: CodexKind::Photograph,
        title: "Wedding, chapel steps",
        text: "A couple on the chapel steps in June light. He is laughing. On the back: Tom and Ada.",
        ..C
    },
    CodexDef {
        id: "photo_constable",
        kind: CodexKind::Photograph,
        title: "Constable in uniform",
        text: "A young constable outside the police station, helmet under his arm. The station keys are on his belt.",
        reveals: Some("station"),
        ..C
    },
    CodexDef {
        id: "photo_choir",
        kind: CodexKind::Photograph,
        title: "The chapel choir",
        text: "Twenty voices in rows. Someone has circled the choirmaster's mouth in ink, over and over, until the paper tore.",
        ..C
    },
    CodexDef {
        id: "photo_engine",
        kind: CodexKind::Photograph,
        title: "The new engine",
        text: "Men in their best coats in front of a flywheel taller than a house. Written below: First steam. Summer.",
        ..C
    },
    CodexDef {
        id: "photo_class",
        kind: CodexKind::Photograph,
        title: "Class photograph",
        text: "Thirty children on benches, the headmistress at the end. Every child has their eyes closed and their mouth a little open.",
        ..C
    },
    CodexDef {
        id: "photo_hale",
        kind: CodexKind::Photograph,
        title: "The doctor and his wife",
        text: "Hale and his wife on a beach somewhere, squinting into wind. Far from any river.",
        ..C
    },
    CodexDef {
        id: "photo_launch",
        kind: CodexKind::Photograph,
        title: "Launch of the Kittiwake",
        text: "The ferry going down the slip, bunting on the rail. A man at the wheel with his hat raised. A woman on the bank with her hands over her ears, smiling.",
        ..C
    },
    CodexDef {
        id: "photo_crowd",
        kind: CodexKind::Photograph,
        title: "Market square, first light",
        text: "Taken from a high window. The square is full. Two hundred people stand in their night clothes, and every one of them faces the mill.",
        ..C
    },
    // Radio
    CodexDef {
        id: "radio_loop",
        kind: CodexKind::Radio,
        title: "Recorded announcement",
        text: "This is a recorded announcement. Remain indoors. Make no noise after dark. Do not attempt to travel by road. Further instructions will follow. This is a recorded announcement.",
        day: 1,
        radio: 1,
        ..C
    },
    CodexDef {
        id: "radio_doctor",
        kind: CodexKind::Radio,
        title: "Station list",
        text: "A man reads names of places and whether anyone has answered. Calder's Reach: no answer. Quarry Lane surgery, Calder's Reach: no answer. He moves on.",
        reveals: Some("surgery"),
        day: 2,
        radio: 1,
    },
    CodexDef {
        id: "radio_saltmouth",
        kind: CodexKind::Radio,
        title: "A woman at Saltmouth",
        text: "This is Saltmouth, at the estuary. We are here. The sound did not reach the sea. If you are upriver, come by water. Not by road. Come by water.",
        day: 3,
        radio: 1,
        ..C
    },
    CodexDef {
        id: "radio_boatyard",
        kind: CodexKind::Radio,
        title: "Saltmouth, again",
        text: "Saltmouth. Anyone upriver with a boat, the boatyards will have pitch and plate. Patch what you have. The tide is with you mornings.",
        reveals: Some("boatyard"),
        day: 5,
        radio: 1,
    },
    CodexDef {
        id: "radio_numbers",
        kind: CodexKind::Radio,
        title: "Numbers",
        text: "A flat voice reading numbers in fives. Four four one, two seven. Then a tone, long and low, and you turn it off before you know you have.",
        day: 6,
        radio: 1,
        ..C
    },
    CodexDef {
        id: "radio_garage",
        kind: CodexKind::Radio,
        title: "A mechanic's question",
        text: "A man asking anyone near Calder's Reach whether Pell's garage still stands, because his engine parts are there. Nobody answers him.",
        reveals: Some("garage"),
        day: 8,
        radio: 2,
    },
    CodexDef {
        id: "radio_ice",
        kind: CodexKind::Radio,
        title: "River warning",
        text: "Saltmouth. The upper river froze last year in the second week of the hard frosts. Expect it sooner this year. Do not wait for the ice.",
        day: 10,
        radio: 1,
        ..C
    },
    CodexDef {
        id: "radio_coal",
        kind: CodexKind::Radio,
        title: "A bargeman",
        text: "A bargeman, drunk or tired, saying he left good fuel at the Calder coal wharf and he would give a lot to have it now.",
        reveals: Some("wharf"),
        day: 11,
        radio: 1,
    },
    CodexDef {
        id: "radio_choir",
        kind: CodexKind::Radio,
        title: "Singing",
        text: "On a frequency that should be empty, voices singing one note, held without breath for longer than anyone could hold it.",
        day: 13,
        radio: 2,
        ..C
    },
    CodexDef {
        id: "radio_farm",
        kind: CodexKind::Radio,
        title: "A shepherd",
        text: "A man on the tops above Calder's Reach. Thirty of us up here, he says, where it is quiet. Hollin's cellar is full. Take what you need.",
        reveals: Some("farm"),
        day: 15,
        radio: 2,
    },
    CodexDef {
        id: "radio_cold",
        kind: CodexKind::Radio,
        title: "Something learned",
        text: "Saltmouth. People upriver report they are slower in the frost. Slower, not stopped. Use the cold days.",
        day: 18,
        radio: 1,
        ..C
    },
    CodexDef {
        id: "radio_last_boats",
        kind: CodexKind::Radio,
        title: "Last boats",
        text: "Saltmouth. We will keep a light on the estuary wall until the ice. After that we cannot help you. Come by water.",
        day: 24,
        radio: 1,
        ..C
    },
    CodexDef {
        id: "radio_quiet",
        kind: CodexKind::Radio,
        title: "Silence",
        text: "Every frequency is empty but one, and on that one somebody is breathing close to the microphone and saying nothing.",
        day: 30,
        radio: 3,
        ..C
    },
    // Field guide
    CodexDef {
        id: "fg_still",
        kind: CodexKind::FieldGuide,
        title: "The unmoving",
        text: "Most do not move at all by day. They stand where the note found them. Noise near them is a risk; noise in the dark is worse.",
        ..C
    },
    CodexDef {
        id: "fg_sound",
        kind: CodexKind::FieldGuide,
        title: "They come to sound",
        text: "They do not see you. They hear you. Every sound you make in a day is remembered by the night.",
        ..C
    },
    CodexDef {
        id: "fg_stander",
        kind: CodexKind::FieldGuide,
        title: "Standers",
        text: "The common kind. Slow to close, slow to turn, strong when they reach you. Back away and they lose the thread.",
        ..C
    },
    CodexDef {
        id: "fg_listener",
        kind: CodexKind::FieldGuide,
        title: "Listeners",
        text: "Head tipped, mouth open. Weak, but they call, and others come. Holding still does not fool them. Deal with them first.",
        ..C
    },
    CodexDef {
        id: "fg_runner",
        kind: CodexKind::FieldGuide,
        title: "Runners",
        text: "They cross a room in a heartbeat. Light, and they fall easily, but they will be on you before you choose.",
        ..C
    },
    CodexDef {
        id: "fg_drowned",
        kind: CodexKind::FieldGuide,
        title: "The drowned",
        text: "They stood in the water when the note came and they are standing there still. Heavy. If they take hold, they do not let go.",
        ..C
    },
    CodexDef {
        id: "fg_hollow",
        kind: CodexKind::FieldGuide,
        title: "Hollows",
        text: "Taller than they were. Their chests have fallen in, as if something was taken out. They come late in the year, and they do not stop.",
        ..C
    },
    CodexDef {
        id: "fg_small",
        kind: CodexKind::FieldGuide,
        title: "Small ones",
        text: "The children heard it first, weeks before. They are quick, and they hum.",
        ..C
    },
    CodexDef {
        id: "fg_cold",
        kind: CodexKind::FieldGuide,
        title: "The cold",
        text: "In frost they are slower to close and slower to wake. Not stopped.",
        ..C
    },
    CodexDef {
        id: "fg_water",
        kind: CodexKind::FieldGuide,
        title: "Running water",
        text: "Loud water drowns the note. They keep away from the weir. The lock house is the quietest place in the valley because it is the loudest.",
        ..C
    },
];

pub fn get(id: &str) -> Option<&'static CodexDef> {
    CODEX.iter().find(|c| c.id == id)
}

pub fn count(kind: CodexKind) -> usize {
    CODEX.iter().filter(|c| c.kind == kind).count()
}
