pub mod stage0;
pub mod stage1;

pub struct HookDisableGuard<T>
where
    T: retour::Function,
{
    enabled: bool,
    detour: &'static retour::StaticDetour<T>,
}

impl<T> HookDisableGuard<T>
where
    T: retour::Function,
{
    pub unsafe fn new(detour: &'static retour::StaticDetour<T>) -> Result<Self, retour::Error> {
        let guard = Self {
            enabled: detour.is_enabled(),
            detour,
        };
        if guard.enabled {
            unsafe { detour.disable()? };
        }
        Ok(guard)
    }
}

impl<T> Drop for HookDisableGuard<T>
where
    T: retour::Function,
{
    fn drop(&mut self) {
        if self.enabled {
            unsafe { self.detour.enable().unwrap() }
        }
    }
}
