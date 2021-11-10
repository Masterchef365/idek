use watertender::prelude::*;
use anyhow::Result;

/// Launch an App
pub fn launch<A: crate::App>(settings: crate::Settings) -> Result<()> {
    let info = AppInfo::default().name(settings.name)?;
    //watertender::starter_kit::launch::<Engine, _>(info, settings.vr, settings)
    todo!()
}

struct Engine {
}
