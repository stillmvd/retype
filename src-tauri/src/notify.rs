use std::sync::Mutex;
use windows::core::{IInspectable, Interface, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Foundation::TypedEventHandler;
use windows::UI::Notifications::{ToastActivatedEventArgs, ToastNotification, ToastNotificationManager};

const AUMID: &str = if cfg!(debug_assertions) { "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe" } else { "com.stillmvd.ghbdtn" };

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

static LAST: Mutex<Vec<ToastNotification>> = Mutex::new(Vec::new());

pub fn reminder(title: &str, body: &str, on_start: impl Fn() + Send + 'static) -> windows::core::Result<()> {
    let xml = XmlDocument::new()?;
    xml.LoadXml(&HSTRING::from(format!(
        "<toast launch=\"start\"><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual><actions><action content=\"Размяться\" arguments=\"start\"/><action content=\"Не сегодня\" arguments=\"skip\"/></actions></toast>",
        escape(title),
        escape(body)
    )))?;
    let toast = ToastNotification::CreateToastNotification(&xml)?;
    toast.Activated(&TypedEventHandler::new(move |_: windows::core::Ref<ToastNotification>, args: windows::core::Ref<IInspectable>| {
        let start = args.as_ref().and_then(|a| a.cast::<ToastActivatedEventArgs>().ok()).and_then(|a| a.Arguments().ok()).is_none_or(|a| a != "skip");
        if start {
            on_start();
        }
        Ok(())
    }))?;
    ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?.Show(&toast)?;
    if let Ok(mut last) = LAST.lock() {
        if last.len() >= 4 {
            last.remove(0);
        }
        last.push(toast);
    }
    Ok(())
}
