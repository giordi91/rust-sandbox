#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum ResourceHandleType {
    Shader = 1,
    Texture= 2,
    Mesh = 3,
    BindingGroup= 4,
    RenderPipeline= 5,
    Invalid = !0,
}

const HANDLE_TYPE_BIT_COUNT: u64 = 10;
//common trick, to generate a mask. If you wanna set the low n bits of an int,
//you shift by N and subtract one, subtracting one will flipp all the N bits to 1
const HANDLE_TYPE_MASK_BIT_RANGE: u64 = (1 << HANDLE_TYPE_BIT_COUNT) - 1;
const HANDLE_TYPE_MASK_FLAG: u64 = HANDLE_TYPE_MASK_BIT_RANGE << (64 - HANDLE_TYPE_BIT_COUNT);

pub struct ResourceHandle {
    data: u64,
}

impl ResourceHandle {
    pub fn new(handle_type: ResourceHandleType, value: u64) -> Self {
        let handle_bits = (handle_type as u64) << (64 - HANDLE_TYPE_BIT_COUNT);
        Self {
            data: (handle_bits | value),
        }
    }
    pub fn from_data(data: u64) -> Self {
        Self { data }
    }

    pub fn get_type(&self) -> ResourceHandleType {
        let handle_type_bits = (self.data & HANDLE_TYPE_MASK_FLAG) >> (64 - HANDLE_TYPE_BIT_COUNT);
        let handle_type_u8 = handle_type_bits as u8;
        let handle_type: ResourceHandleType = unsafe { std::mem::transmute(handle_type_u8) };
        handle_type
    }
    pub fn get_value(&self) -> u64 {
        self.data & (!HANDLE_TYPE_MASK_FLAG)
    }

    pub fn raw(&self) -> u64 {
        self.data
    }
}
