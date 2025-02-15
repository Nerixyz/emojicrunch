use std::{marker::PhantomData, mem::MaybeUninit};

pub struct Options<'a> {
    inner: zopflipng_sys::CZopfliPNGOptions,
    _marker: PhantomData<&'a ()>,
}

unsafe impl Send for Options<'_> {}
unsafe impl Sync for Options<'_> {}

pub type FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy;

pub const STRATEGY_ZERO: FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyZero;
pub const STRATEGY_ONE: FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyOne;
pub const STRATEGY_TWO: FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyTwo;
pub const STRATEGY_THREE: FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyThree;
pub const STRATEGY_FOUR: FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyFour;
pub const STRATEGY_MIN_SUM: FilterStrategy = zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyMinSum;
pub const STRATEGY_ENTROPY: FilterStrategy =
    zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyEntropy;
pub const STRATEGY_PREDEFINED: FilterStrategy =
    zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyPredefined;
pub const STRATEGY_BRUTE_FORCE: FilterStrategy =
    zopflipng_sys::ZopfliPNGFilterStrategy_kStrategyBruteForce;

impl<'a> Options<'a> {
    pub fn new() -> Self {
        let mut inner = MaybeUninit::uninit();
        unsafe {
            zopflipng_sys::CZopfliPNGSetDefaults(inner.as_mut_ptr());
        }
        Self {
            inner: unsafe { inner.assume_init() },
            _marker: PhantomData,
        }
    }

    pub fn set_lossy_transparent(&mut self, value: bool) -> &mut Self {
        self.inner.lossy_transparent = if value { 1 } else { 0 };
        self
    }

    pub fn set_num_iterations(&mut self, value: i32) -> &mut Self {
        self.inner.num_iterations = value;
        self
    }

    pub fn set_num_iterations_large(&mut self, value: i32) -> &mut Self {
        self.inner.num_iterations_large = value;
        self
    }

    pub fn set_filter_strategies(&mut self, strategies: &'a [FilterStrategy]) -> &mut Self {
        self.inner.filter_strategies = strategies.as_ptr() as *mut _;
        self.inner.num_filter_strategies = strategies.len() as std::ffi::c_int;
        self
    }
}

pub struct OptimizedPng {
    data: *const u8,
    size: usize,
}

impl Drop for OptimizedPng {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.data as *mut _);
        }
    }
}

impl AsRef<[u8]> for OptimizedPng {
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.size) }
    }
}

pub fn optimize(data: &[u8], options: &Options) -> Result<OptimizedPng, i32> {
    unsafe {
        let mut out = std::ptr::null_mut();
        let mut out_size = 0;
        match zopflipng_sys::CZopfliPNGOptimize(
            data.as_ptr(),
            data.len(),
            &options.inner,
            0,
            &mut out,
            &mut out_size,
        ) {
            0 => Ok(OptimizedPng {
                data: out,
                size: out_size,
            }),
            x => Err(x),
        }
    }
}
