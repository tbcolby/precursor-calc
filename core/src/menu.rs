//! The three F-key menus. Each item is chosen by a single key.

use crate::functions::Func;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuId {
    Math,
    Trig,
    More,
}

impl MenuId {
    /// Menu opened by F1..F3.
    pub fn from_fkey(n: u8) -> Option<MenuId> {
        Some(match n {
            1 => MenuId::Math,
            2 => MenuId::Trig,
            3 => MenuId::More,
            _ => return None,
        })
    }

    pub fn title(self) -> &'static str {
        match self {
            MenuId::Math => "F1 Math",
            MenuId::Trig => "F2 Trig",
            MenuId::More => "F3 More",
        }
    }

    pub fn items(self) -> &'static [MenuItem] {
        match self {
            MenuId::Math => MATH,
            MenuId::Trig => TRIG,
            MenuId::More => MORE,
        }
    }

    /// The item bound to `key`, if any.
    pub fn find(self, key: char) -> Option<MenuAction> {
        self.items().iter().find(|i| i.key == key).map(|i| i.action)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Func(Func),
    ChangeSign,
    Store,
    Recall,
    ClearMemory,
    ToggleMode,
    CycleAngle,
    CycleBase,
    /// Clear the "saved data unreadable" lock and allow saving again.
    OverwriteSaved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub key: char,
    pub label: &'static str,
    pub action: MenuAction,
}

const fn f(key: char, func: Func) -> MenuItem { MenuItem { key, label: "", action: MenuAction::Func(func) } }

const fn item(key: char, label: &'static str, action: MenuAction) -> MenuItem { MenuItem { key, label, action } }

impl MenuItem {
    /// Display label (function items use the function's own label).
    pub fn text(&self) -> &'static str {
        match self.action {
            MenuAction::Func(func) if self.label.is_empty() => func.label(),
            _ => self.label,
        }
    }
}

static MATH: &[MenuItem] = &[
    f('1', Func::Sqrt),
    f('2', Func::Square),
    f('3', Func::Reciprocal),
    f('4', Func::Ln),
    f('5', Func::Log),
    f('6', Func::Exp),
    f('7', Func::Exp10),
    f('8', Func::Factorial),
    f('9', Func::Pi),
    f('0', Func::E),
];

static TRIG: &[MenuItem] = &[
    f('1', Func::Sin),
    f('2', Func::Cos),
    f('3', Func::Tan),
    f('4', Func::Asin),
    f('5', Func::Acos),
    f('6', Func::Atan),
    f('7', Func::Sinh),
    f('8', Func::Cosh),
    f('9', Func::Tanh),
    f('0', Func::Asinh),
];

static MORE: &[MenuItem] = &[
    f('1', Func::Abs),
    f('2', Func::Floor),
    f('3', Func::Ceil),
    f('4', Func::Round),
    f('5', Func::Cbrt),
    f('6', Func::Cube),
    f('7', Func::Log2),
    item('8', "+/- (change sign)", MenuAction::ChangeSign),
    item('9', "STO to M0-M9", MenuAction::Store),
    item('0', "RCL from M0-M9", MenuAction::Recall),
    item('m', "ALG / RPN", MenuAction::ToggleMode),
    item('a', "DEG / RAD / GRAD", MenuAction::CycleAngle),
    item('b', "DEC / HEX / OCT / BIN", MenuAction::CycleBase),
    item('c', "Clear memory M0-M9", MenuAction::ClearMemory),
    item('w', "Overwrite unreadable save", MenuAction::OverwriteSaved),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_per_menu() {
        for m in [MenuId::Math, MenuId::Trig, MenuId::More] {
            let items = m.items();
            for (i, a) in items.iter().enumerate() {
                assert!(!a.text().is_empty());
                for b in &items[i + 1..] {
                    assert_ne!(a.key, b.key, "{:?}", m);
                }
            }
        }
    }

    #[test]
    fn find_uses_pressed_key() {
        assert_eq!(MenuId::Math.find('1'), Some(MenuAction::Func(Func::Sqrt)));
        assert_eq!(MenuId::More.find('0'), Some(MenuAction::Recall));
        assert_eq!(MenuId::Trig.find('z'), None);
        assert_eq!(MenuId::from_fkey(4), None, "F4 is Back, never a menu");
    }
}
