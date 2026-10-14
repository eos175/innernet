#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Disable DMA-BUF renderer on Linux to prevent WebKitGTK DRM_IOCTL_MODE_CREATE_DUMB permission errors
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    innernet_desktop::run();
}
