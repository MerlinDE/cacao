//! Implements a wrapper type for `NSColor` and `UIColor`. It attempts to map 
//! to a common shared API, but it's important to note that the platforms 
//! themselves have differing levels of support for color work. Where possible,
//! we expose some platform-specific methods for creating and working with these.
//!
//! We attempt to provide fallbacks for older versions of macOS/iOS, but this is not exhaustive,
/// as the cross-section of people building for older platforms in Rust is likely very low. If you
/// need these fallbacks to be better and/or correct, you're welcome to improve and pull-request
/// this.
///
/// The goal here is to make sure that this can't reasonably break on OS's, as `Color` is kind of
/// an important piece. It's not on the framework to make your app look good, though. To enable
/// fallbacks, specify the `color_fallbacks` feature in your `Cargo.toml`.
///
/// @TODO: bundle iOS/tvOS support.

use core_graphics::base::CGFloat;
use core_graphics::color::CGColor;

use objc::{class, msg_send, sel, sel_impl};
use objc::runtime::Object;
use objc_id::ShareId;

use crate::foundation::id;
use crate::utils::os;

#[cfg(feature = "macos")]
mod macos_dynamic_color; 

#[cfg(feature = "macos")]
use macos_dynamic_color::{
    AQUA_LIGHT_COLOR_NORMAL_CONTRAST, AQUA_LIGHT_COLOR_HIGH_CONTRAST,
    AQUA_DARK_COLOR_NORMAL_CONTRAST, AQUA_DARK_COLOR_HIGH_CONTRAST
};

/// Represents a rendering style - dark mode or light mode.
/// In the event that a new variant is introduced in later versions of
/// macOS or iOS, calls that use the dynamic color(s) from here will likely
/// default to the `Light` theme.
#[derive(Debug)]
pub enum Theme {
    /// The "default" theme on a platform. On macOS, this is Aqua.
    /// On iOS and tvOS, this is whatever you call the system defined theme.
    Light,

    /// Dark mode.
    Dark
}

/// Represents the contrast level for a rendering context. 
#[derive(Debug)]
pub enum Contrast {
    /// The default contrast level for the system.
    Normal,

    /// The high contrast level for the system.
    High
}

/// A `Style` is passed to you when doing dynamic color calculations. You can opt to
/// provide different colors depending on the settings in here - notably, this is useful
/// for supporting dark mode and high contrast accessibility contexts.
#[derive(Debug)]
pub struct Style {
    /// Represents the current theme for where this color may render.
    pub theme: Theme,

    /// Represents the current contrast level for where this color may render.
    pub contrast: Contrast
}

/*
#[derive(Clone)]
pub struct Property(Rc<RefCell<Id<Object>>>);

impl Property {
    pub fn new(obj: id) -> Self {
        Property(Rc::new(RefCell::new(Id::from_ptr(obj))))
    }
}

#[derive(Clone)]
pub struct ThreadSafeProperty(Arc<RwLock<Id<Object>>>);

impl Property {
    pub fn new(obj: id) -> Self {
        Property(Rc::new(RefCell::new(Id::from_ptr(obj))))
    }
}
*/

/// Represents a Color. You can create custom colors using the various 
/// initializers, or opt to use a system-provided color. The system provided
/// colors will automatically switch to the "correct" colors/shades depending on whether
/// the user is in light or dark mode; to support this with custom colors, be sure
/// to call the `.dark()` method after initializing.
#[derive(Clone)]
pub enum Color {
    /// Represents an `NSColor` on macOS, and a `UIColor` everywhere else. You typically
    /// don't create this variant yourself; use the initializers found on this enum.
    ///
    /// If you need to do custom work not covered by this enum, you can drop to 
    /// the Objective-C level yourself and wrap your color in this.
    Object(ShareId<Object>),

    /// The system-provided black. Harsh - you probably don't want to use this.
    SystemBlack,

    /// The system-provided absolute white.
    SystemWhite,

    /// The system-provided brown.
    SystemBrown,

    /// The system-provided blue.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemBlue,

    /// The system-provided green.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGreen,

    /// The system-provided indigo.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemIndigo,

    /// The system-provided orange.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemOrange,

    /// The system-provided pink.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemPink,

    /// The system-provided purple.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemPurple,

    /// The system-provided red.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemRed,

    /// The system-provided teal.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemTeal,

    /// The system-provided yellow.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemYellow,

    /// The system-provided base gray color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGray,
    
    /// The system-provided secondary-level gray color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGray2,

    /// The system-provided third-level gray color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGray3,

    /// The system-provided fourth-level gray color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGray4,
    
    /// The system-provided fifth-level gray color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGray5,
    
    /// The system-provided sixth-level gray color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemGray6,

    /// Represents a clear color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    Clear,

    /// The default label color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    Label,

    /// The default color for a second-level label.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    LabelSecondary,

    /// The default color for a third-level label.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    LabelTertiary,

    /// The default color for a fourth-level label.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    LabelQuaternary,

    /// The default system fill color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemFill,

    /// The default system second-level fill color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemFillSecondary,
    
    /// The default system third-level fill color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemFillTertiary,
    
    /// The default system fourth-level fill color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemFillQuaternary,

    /// The default color to use for placeholder text.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    PlaceholderText,

    /// The default system background color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemBackground,

    /// The default system second-level background color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemBackgroundSecondary,
    
    /// The default system third-level background color.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    SystemBackgroundTertiary,

    /// The default color to use for thin separators/lines that
    /// still allow content to be visible underneath.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    Separator,

    /// The default color to use for thin separators/lines that
    /// do not allow content underneath to be visible.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    OpaqueSeparator,

    /// The default color to use for rendering links.
    /// This value automatically switches to the correct variant depending on light or dark mode.
    Link,

    /// The un-adaptable color for text on a light background.
    DarkText,

    /// The un-adaptable color for text on a dark background.
    LightText
}

impl Color {
    /// Creates and returns a color in the RGB space, with the specified
    /// alpha level.
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        let r = red as CGFloat / 255.0;
        let g = green as CGFloat / 255.0;
        let b = blue as CGFloat / 255.0;
        let a = alpha as CGFloat / 255.0;
        
        #[cfg(feature = "macos")]
        let color = class!(NSColor);

        #[cfg(feature = "ios")]
        let color = class!(UIColor);

        Color::Object(unsafe {
            #[cfg(feature = "macos")]
            ShareId::from_ptr(msg_send![color, colorWithCalibratedRed:r green:g blue:b alpha:a])
        })
    }
    
    /// Creates and returns a color in the RGB space, with the alpha level
    /// set to `255` by default. Shorthand for `rgba`.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Color::rgba(red, green, blue, 255)
    }

    /// Creates and returns a color in the HSB space, with the specified
    /// alpha level.
    pub fn hsba(hue: u8, saturation: u8, brightness: u8, alpha: u8) -> Self {
        let h = hue as CGFloat / 255.0;
        let s = saturation as CGFloat / 255.0;
        let b = brightness as CGFloat / 255.0;
        let a = alpha as CGFloat / 255.0;

        #[cfg(feature = "macos")]
        let color = class!(NSColor);

        #[cfg(feature = "ios")]
        let color = class!(UIColor);

        Color::Object(unsafe {
            #[cfg(feature = "macos")]
            ShareId::from_ptr(msg_send![color, colorWithCalibratedHue:h saturation:s brightness:b alpha:a])
        })
    }
    
    /// Creates and returns a color in the RGB space, with the alpha level
    /// set to `255` by default. Shorthand for `hsba`.
    pub fn hsb(hue: u8, saturation: u8, brightness: u8) -> Self {
        Color::hsba(hue, saturation, brightness, 255)
    }

    /// Creates and returns a white color with the specified level or intensity, along with the
    /// specified alpha.
    pub fn white_alpha(level: CGFloat, alpha: CGFloat) -> Self {
        #[cfg(feature = "macos")]
        let color = class!(NSColor);

        #[cfg(feature = "ios")]
        let color = class!(UIColor);

        Color::Object(unsafe {
            #[cfg(feature = "macos")]
            ShareId::from_ptr(msg_send![color, colorWithCalibratedWhite:level alpha:alpha])
        })
    }

    /// Creates and returns a white Color with the specified level or intensity, with the alpha
    /// value set to `255`. Shorthand for `white_alpha`.
    pub fn white(level: CGFloat) -> Self {
        Color::white_alpha(level, 1.0)
    }
    
    /// Given a hex code and alpha level, returns a `Color` in the RGB space.
    ///
    /// This method is not an ideal one to use, but is offered as a convenience method for those
    /// coming from other environments where these are more common.
    pub fn hexa(hex: &str, alpha: u8) -> Self {
        Color::SystemRed
    }

    /// Given a hex code, returns a `Color` in the RGB space with alpha pre-set to `255`.
    ///
    /// This method is not an ideal one to use, but is offered as a convenience method for those
    /// coming from other environments where these are more common.
    pub fn hex(hex: &str) -> Self {
        Color::hexa(hex, 255)
    }

    /// Creates and returns a dynamic color, which stores a handler and enables returning specific
    /// colors at appearance time based on device traits (i.e, dark mode vs light mode, contrast
    /// settings, etc).
    ///
    /// For systems that don't support dark mode (macOS pre-Mojave) this will always paint with the
    /// "default" or "light" color.
    ///
    /// Returning a dynamic color in your handler is unsupported and may panic.
    pub fn dynamic<F>(handler: F) -> Self
    where
        F: Fn(Style) -> Color + 'static
    {
        #[cfg(feature = "macos")]
        Color::Object(unsafe {
            let color: id = msg_send![macos_dynamic_color::register_class(), new];

            (&mut *color).set_ivar(AQUA_LIGHT_COLOR_NORMAL_CONTRAST, handler(Style {
                theme: Theme::Light,
                contrast: Contrast::Normal
            }).to_objc());

            (&mut *color).set_ivar(AQUA_LIGHT_COLOR_HIGH_CONTRAST, handler(Style {
                theme: Theme::Light,
                contrast: Contrast::High
            }).to_objc());

            (&mut *color).set_ivar(AQUA_DARK_COLOR_NORMAL_CONTRAST, handler(Style {
                theme: Theme::Dark,
                contrast: Contrast::Normal
            }).to_objc());

            (&mut *color).set_ivar(AQUA_DARK_COLOR_HIGH_CONTRAST, handler(Style {
                theme: Theme::Light,
                contrast: Contrast::Normal
            }).to_objc());
            
            ShareId::from_ptr(color)
        })
    }

    /// Returns a pointer that can be used for the Objective-C runtime.
    /// 
    /// This method is primarily for internal use, but is kept public for those who might need to
    /// work with colors outside of what's available in this enum.
    pub fn to_objc(&self) -> id {
        unsafe { to_objc(self) }
    }
    
    /// Legacy.
    pub fn into_platform_specific_color(&self) -> id {
        unsafe { to_objc(self) }
    }

    /// Returns a CGColor, which can be used in Core Graphics calls as well as other areas.
    ///
    /// Note that CGColor is _not_ a context-aware color, unlike our `NSColor` and `UIColor`
    /// objects. If you're painting in a context that requires dark mode support, make sure 
    /// you're not using a cached version of this unless you explicitly want the _same_ color
    /// in every context it's used in.
    pub fn cg_color(&self) -> CGColor {
        // @TODO: This should probably return a CGColorRef...
        unsafe {
            let objc = to_objc(self);
            msg_send![objc, CGColor]
        }
    }
}

impl AsRef<Color> for Color {
    /// Provided to make passing `Color` types around less of a headache.
    #[inline]
    fn as_ref(&self) -> &Color {
        self
    }
}

/// Handles color fallback for system-provided colors.
macro_rules! system_color_with_fallback {
    ($class:ident, $color:ident, $fallback:ident) => ({
        #[cfg(feature = "macos")]
        {
            #[cfg(feature = "color_fallbacks")]
            if os::minimum_semversion(10, 10, 0) {
                msg_send![$class, $color]
            } else {
                msg_send![$class, $fallback]
            }

            #[cfg(not(feature = "color_fallbacks"))]
            msg_send![$class, $color]
        }
    })
}

/// This function maps enum types to system-provided colors, or our stored NS/UIColor objects.
/// It attempts to provide fallbacks for older versions of macOS/iOS, but this is not exhaustive,
/// as the cross-section of people building for older platforms in Rust is likely very low. If you
/// need these fallbacks to be better and/or correct, you're welcome to improve and pull-request
/// this.
///
/// The goal here is to make sure that this can't reasonably break on OS's, as `Color` is kind of
/// an important piece. It's not on the framework to make your app look good, though.
unsafe fn to_objc(obj: &Color) -> id {
    #[cfg(feature = "macos")]
    let color = class!(NSColor);

    #[cfg(feature = "ios")]
    let color = class!(UIColor);

    match obj {
        // Regardless of platform, we can just dereference this one.
        Color::Object(obj) => msg_send![&**obj, self],

        Color::SystemBlack => msg_send![color, blackColor],
        Color::SystemWhite => msg_send![color, whiteColor],
        Color::SystemBrown => msg_send![color, brownColor],
        Color::SystemBlue => system_color_with_fallback!(color, systemBlueColor, blueColor),
        Color::SystemGreen => system_color_with_fallback!(color, systemGreenColor, greenColor),
        Color::SystemIndigo => system_color_with_fallback!(color, systemIndigoColor, magentaColor),
        Color::SystemOrange => system_color_with_fallback!(color, systemOrangeColor, orangeColor),
        Color::SystemPink => system_color_with_fallback!(color, systemPinkColor, pinkColor), 
        Color::SystemPurple => system_color_with_fallback!(color, systemPurpleColor, purpleColor),
        Color::SystemRed => system_color_with_fallback!(color, systemRedColor, redColor),
        Color::SystemTeal => system_color_with_fallback!(color, systemTealColor, blueColor),
        Color::SystemYellow => system_color_with_fallback!(color, systemYellowColor, yellowColor),
        Color::SystemGray => system_color_with_fallback!(color, systemGrayColor, darkGrayColor),
        Color::SystemGray2 => system_color_with_fallback!(color, systemGray2Color, grayColor),
        Color::SystemGray3 => system_color_with_fallback!(color, systemGray3Color, lightGrayColor),
        Color::SystemGray4 => system_color_with_fallback!(color, systemGray4Color, lightGrayColor),
        Color::SystemGray5 => system_color_with_fallback!(color, systemGray5Color, lightGrayColor),
        Color::SystemGray6 => system_color_with_fallback!(color, systemGray6Color, lightGrayColor),
        Color::Clear => msg_send![color, clearColor],
        Color::Label => system_color_with_fallback!(color, labelColor, blackColor),
        Color::LabelSecondary => system_color_with_fallback!(color, secondaryLabelColor, blackColor),
        Color::LabelTertiary => system_color_with_fallback!(color, tertiaryLabelColor, blackColor),
        Color::LabelQuaternary => system_color_with_fallback!(color, quaternaryLabelColor, blackColor),
        Color::SystemFill => system_color_with_fallback!(color, systemFillColor, clearColor),
        Color::SystemFillSecondary => system_color_with_fallback!(color, secondarySystemFillColor, clearColor),
        Color::SystemFillTertiary => system_color_with_fallback!(color, tertiarySystemFillColor, clearColor),
        Color::SystemFillQuaternary => system_color_with_fallback!(color, quaternarySystemFillColor, clearColor),
        Color::PlaceholderText => system_color_with_fallback!(color, placeholderTextColor, darkGrayColor),
        Color::SystemBackground => system_color_with_fallback!(color, systemBackgroundColor, clearColor),
        Color::SystemBackgroundSecondary => system_color_with_fallback!(color, secondarySystemBackgroundColor, clearColor),
        Color::SystemBackgroundTertiary => system_color_with_fallback!(color, tertiarySystemBackgroundColor, clearColor),
        Color::Separator => system_color_with_fallback!(color, separatorColor, lightGrayColor),
        Color::OpaqueSeparator => system_color_with_fallback!(color, opaqueSeparatorColor, darkGrayColor),
        Color::Link => system_color_with_fallback!(color, linkColor, blueColor),
        Color::DarkText => system_color_with_fallback!(color, darkTextColor, blackColor),
        Color::LightText => system_color_with_fallback!(color, lightTextColor, whiteColor),
    }
}
