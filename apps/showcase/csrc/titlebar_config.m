// Configure the native macOS titlebar for Chrome-style tabs-in-titlebar:
//   1. Make the titlebar fully transparent (hide visual effect backgrounds)
//   2. Swizzle hitTest: so only traffic-light buttons intercept clicks;
//      all other clicks pass through to iced's content view (tab bar)
//   3. Center traffic-light buttons vertically within the tab bar height
//
// Must be called on the main thread AFTER iced/winit has realized the window.

#import <Cocoa/Cocoa.h>
#import <objc/runtime.h>

// ---------------------------------------------------------------------------
// hitTest: swizzle — only let traffic-light buttons receive events
// ---------------------------------------------------------------------------

static IMP g_original_hitTest = NULL;

// Check whether a view is (or is an ancestor of) a standard window button.
static BOOL is_window_button(NSView *view) {
    while (view) {
        if ([view isKindOfClass:[NSButton class]]) {
            // Standard traffic-light buttons are _NSThemeWidget subclasses
            // inside the NSTitlebarView. Any NSButton inside the titlebar
            // container is a window control we want to keep interactive.
            return YES;
        }
        view = view.superview;
    }
    return NO;
}

static NSView *swizzled_hitTest(id self, SEL _cmd, NSPoint point) {
    // Call original to find the deepest hit view.
    NSView *hit = ((NSView *(*)(id, SEL, NSPoint))g_original_hitTest)(self, _cmd, point);
    if (!hit) return nil;

    // Allow hits on traffic-light buttons (close, minimize, zoom).
    if (is_window_button(hit)) return hit;

    // Everything else: return nil → event falls through to the content view.
    return nil;
}

// Install the hitTest: swizzle on the given view's class (once).
static void install_hitTest_swizzle(NSView *container) {
    Class cls = [container class];
    SEL sel = @selector(hitTest:);
    Method m = class_getInstanceMethod(cls, sel);
    if (!m) return;

    // Only swizzle once (idempotent across multiple windows).
    if (g_original_hitTest) return;
    g_original_hitTest = method_getImplementation(m);
    method_setImplementation(m, (IMP)swizzled_hitTest);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// Find the NSTitlebarContainerView in the private view hierarchy.
static NSView *find_titlebar_container(NSWindow *window) {
    NSView *theme_frame = window.contentView.superview;
    if (!theme_frame) return nil;
    for (NSView *sub in theme_frame.subviews) {
        if ([NSStringFromClass([sub class]) containsString:@"NSTitlebarContainerView"]) {
            return sub;
        }
    }
    return nil;
}

// Recursively hide all NSVisualEffectView instances under `root`.
static void hide_visual_effects(NSView *root) {
    for (NSView *sub in root.subviews) {
        if ([sub isKindOfClass:[NSVisualEffectView class]]) {
            sub.hidden = YES;
        }
        hide_visual_effects(sub);
    }
}

// Center the three standard window buttons vertically within `bar_height`.
static void center_traffic_lights(NSWindow *window, CGFloat bar_height) {
    NSButton *buttons[] = {
        [window standardWindowButton:NSWindowCloseButton],
        [window standardWindowButton:NSWindowMiniaturizeButton],
        [window standardWindowButton:NSWindowZoomButton],
    };
    for (int i = 0; i < 3; i++) {
        NSButton *btn = buttons[i];
        if (!btn) continue;
        NSRect f = btn.frame;
        // Center vertically: put the button midpoint at bar_height / 2.
        // Button parent (NSTitlebarView) is inside the container which may
        // be flipped — check and compute accordingly.
        NSView *parent = btn.superview;
        if (parent && parent.isFlipped) {
            f.origin.y = (bar_height - f.size.height) / 2.0;
        } else {
            f.origin.y = (bar_height - f.size.height) / 2.0;
        }
        btn.frame = f;
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

// Configure the titlebar of the window whose title matches `title_utf8`.
//   bar_height — tab bar height in points (traffic lights will center within this)
//   bg_r/g/b   — window background color (from theme surface) to blend the border
// Returns 1 on success, 0 if window or titlebar view not found.
int rl_showcase_configure_titlebar(const char *title_utf8, float bar_height,
                                   float bg_r, float bg_g, float bg_b) {
    if (!title_utf8) return 0;

    NSString *needle = [NSString stringWithUTF8String:title_utf8];
    NSWindow *match = nil;
    for (NSWindow *w in [NSApp windows]) {
        if ([[w title] isEqualToString:needle]) { match = w; break; }
    }
    if (!match) return 0;

    NSView *container = find_titlebar_container(match);
    if (!container) return 0;

    // 1. Transparency: hide all visual effect views inside the titlebar.
    hide_visual_effects(container);

    // 2. Hit-test passthrough: swizzle hitTest: so only buttons respond.
    install_hitTest_swizzle(container);

    // 3. Stretch the container height to match the tab bar and keep it
    //    full-width (so buttons stay in their normal X positions).
    NSRect frame = container.frame;
    CGFloat old_top = frame.origin.y + frame.size.height;
    frame.size.height = bar_height;
    frame.origin.y = old_top - bar_height;
    container.frame = frame;
    // Pin to top, stretch horizontally with window.
    container.autoresizingMask = NSViewWidthSizable | NSViewMinYMargin;

    // 4. Center traffic-light buttons vertically within the new height.
    center_traffic_lights(match, bar_height);

    // 5. Prevent the window from being movable by the titlebar background
    //    (tab bar handles window drag via iced's window::drag).
    match.movable = NO;

    // 6. Soften the native window border. The system derives the 1px stroke
    //    color from the window's backgroundColor. Match it to the theme's
    //    surface color so the border blends in.
    match.backgroundColor = [NSColor colorWithSRGBRed:bg_r green:bg_g blue:bg_b alpha:1.0];

    return 1;
}
