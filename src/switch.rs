//! A wrapper for NSSwitch. Currently the epitome of jank - if you're poking around here, expect
//! that this will change at some point.

use std::fmt;
use std::sync::Once;

use objc_id::ShareId;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::foundation::{id, nil, BOOL, YES, NO, NSString};
use crate::invoker::TargetActionHandler;
use crate::layout::{Layout, LayoutAnchorX, LayoutAnchorY, LayoutAnchorDimension};
use crate::utils::load;

/// A wrapper for `NSSwitch`. Holds (retains) pointers for the Objective-C runtime 
/// where our `NSSwitch` lives.
#[derive(Debug)]
pub struct Switch {
    pub objc: ShareId<Object>,
    handler: Option<TargetActionHandler>,
    
    /// A pointer to the Objective-C runtime top layout constraint.
    pub top: LayoutAnchorY,

    /// A pointer to the Objective-C runtime leading layout constraint.
    pub leading: LayoutAnchorX,

    /// A pointer to the Objective-C runtime trailing layout constraint.
    pub trailing: LayoutAnchorX,

    /// A pointer to the Objective-C runtime bottom layout constraint.
    pub bottom: LayoutAnchorY,

    /// A pointer to the Objective-C runtime width layout constraint.
    pub width: LayoutAnchorDimension,

    /// A pointer to the Objective-C runtime height layout constraint.
    pub height: LayoutAnchorDimension,

    /// A pointer to the Objective-C runtime center X layout constraint.
    pub center_x: LayoutAnchorX,

    /// A pointer to the Objective-C runtime center Y layout constraint.
    pub center_y: LayoutAnchorY
}

impl Switch {
    /// Creates a new `NSSwitch` instance, configures it appropriately,
    /// and retains the necessary Objective-C runtime pointer.
    pub fn new(text: &str) -> Self {
        let title = NSString::new(text);

        let view: id = unsafe {
            let button: id = msg_send![register_class(), buttonWithTitle:title target:nil action:nil];
            let _: () = msg_send![button, setTranslatesAutoresizingMaskIntoConstraints:NO];
            let _: () = msg_send![button, setButtonType:3];
            button
        };
        
        Switch {
            handler: None,
            top: LayoutAnchorY::new(unsafe { msg_send![view, topAnchor] }),
            leading: LayoutAnchorX::new(unsafe { msg_send![view, leadingAnchor] }),
            trailing: LayoutAnchorX::new(unsafe { msg_send![view, trailingAnchor] }),
            bottom: LayoutAnchorY::new(unsafe { msg_send![view, bottomAnchor] }),
            width: LayoutAnchorDimension::new(unsafe { msg_send![view, widthAnchor] }),
            height: LayoutAnchorDimension::new(unsafe { msg_send![view, heightAnchor] }),
            center_x: LayoutAnchorX::new(unsafe { msg_send![view, centerXAnchor] }),
            center_y: LayoutAnchorY::new(unsafe { msg_send![view, centerYAnchor] }),
            objc: unsafe { ShareId::from_ptr(view) },
        }
    }

    /// Sets whether this is checked on or off.
    pub fn set_checked(&mut self, checked: bool) {
        unsafe {
            // @TODO: The constants to use here changed back in 10.13ish, so... do we support that,
            // or just hide it?
            let _: () = msg_send![&*self.objc, setState:match checked {
                true => 1,
                false => 0
            }];
        }
    }

    /// Attaches a callback for button press events. Don't get too creative now...
    /// best just to message pass or something.
    pub fn set_action<F: Fn() + Send + Sync + 'static>(&mut self, action: F) {
        let handler = TargetActionHandler::new(&*self.objc, action);
        self.handler = Some(handler);
    }
}

impl Layout for Switch {
    fn get_backing_node(&self) -> ShareId<Object> {
        self.objc.clone()
    }

    fn add_subview<V: Layout>(&self, _view: &V) { 
        panic!(r#"
            Tried to add a subview to a Button. This is not allowed in Cacao. If you think this should be supported, 
            open a discussion on the GitHub repo.
        "#);    
    }
}

impl Drop for Switch {
    // Just to be sure, let's... nil these out. They should be weak references,
    // but I'd rather be paranoid and remove them later.
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![&*self.objc, setTarget:nil];
            let _: () = msg_send![&*self.objc, setAction:nil];
        }
    }
}

/// Registers an `NSButton` subclass, and configures it to hold some ivars 
/// for various things we need to store.
fn register_class() -> *const Class {
    static mut VIEW_CLASS: *const Class = 0 as *const Class;
    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        let superclass = class!(NSButton);
        let decl = ClassDecl::new("RSTSwitch", superclass).unwrap(); 
        VIEW_CLASS = decl.register();
    });

    unsafe { VIEW_CLASS }
}
