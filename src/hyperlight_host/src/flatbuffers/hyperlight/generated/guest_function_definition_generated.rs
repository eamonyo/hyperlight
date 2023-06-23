// automatically generated by the FlatBuffers compiler, do not modify
// @generated
extern crate alloc;
extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};
use super::*;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem;
pub enum GuestFunctionDefinitionOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct GuestFunctionDefinition<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for GuestFunctionDefinition<'a> {
    type Inner = GuestFunctionDefinition<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table::new(buf, loc),
        }
    }
}

impl<'a> GuestFunctionDefinition<'a> {
    pub const VT_FUNCTION_NAME: flatbuffers::VOffsetT = 4;
    pub const VT_PARAMETERS: flatbuffers::VOffsetT = 6;
    pub const VT_RETURN_TYPE: flatbuffers::VOffsetT = 8;
    pub const VT_FUNCTION_POINTER: flatbuffers::VOffsetT = 10;

    #[inline]
    pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        GuestFunctionDefinition { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args GuestFunctionDefinitionArgs<'args>,
    ) -> flatbuffers::WIPOffset<GuestFunctionDefinition<'bldr>> {
        let mut builder = GuestFunctionDefinitionBuilder::new(_fbb);
        builder.add_function_pointer(args.function_pointer);
        if let Some(x) = args.parameters {
            builder.add_parameters(x);
        }
        if let Some(x) = args.function_name {
            builder.add_function_name(x);
        }
        builder.add_return_type(args.return_type);
        builder.finish()
    }

    #[inline]
    pub fn function_name(&self) -> &'a str {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(
                    GuestFunctionDefinition::VT_FUNCTION_NAME,
                    None,
                )
                .unwrap()
        }
    }
    #[inline]
    pub fn key_compare_less_than(&self, o: &GuestFunctionDefinition) -> bool {
        self.function_name() < o.function_name()
    }

    #[inline]
    pub fn key_compare_with_value(&self, val: &str) -> ::core::cmp::Ordering {
        let key = self.function_name();
        key.cmp(val)
    }
    #[inline]
    pub fn parameters(&self) -> flatbuffers::Vector<'a, ParameterType> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, ParameterType>>>(
                    GuestFunctionDefinition::VT_PARAMETERS,
                    None,
                )
                .unwrap()
        }
    }
    #[inline]
    pub fn return_type(&self) -> ReturnType {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<ReturnType>(
                    GuestFunctionDefinition::VT_RETURN_TYPE,
                    Some(ReturnType::hlint),
                )
                .unwrap()
        }
    }
    #[inline]
    pub fn function_pointer(&self) -> i64 {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<i64>(GuestFunctionDefinition::VT_FUNCTION_POINTER, Some(0))
                .unwrap()
        }
    }
}

impl flatbuffers::Verifiable for GuestFunctionDefinition<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        use self::flatbuffers::Verifiable;
        v.visit_table(pos)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "function_name",
                Self::VT_FUNCTION_NAME,
                true,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, ParameterType>>>(
                "parameters",
                Self::VT_PARAMETERS,
                true,
            )?
            .visit_field::<ReturnType>("return_type", Self::VT_RETURN_TYPE, false)?
            .visit_field::<i64>("function_pointer", Self::VT_FUNCTION_POINTER, false)?
            .finish();
        Ok(())
    }
}
pub struct GuestFunctionDefinitionArgs<'a> {
    pub function_name: Option<flatbuffers::WIPOffset<&'a str>>,
    pub parameters: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, ParameterType>>>,
    pub return_type: ReturnType,
    pub function_pointer: i64,
}
impl<'a> Default for GuestFunctionDefinitionArgs<'a> {
    #[inline]
    fn default() -> Self {
        GuestFunctionDefinitionArgs {
            function_name: None, // required field
            parameters: None,    // required field
            return_type: ReturnType::hlint,
            function_pointer: 0,
        }
    }
}

pub struct GuestFunctionDefinitionBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> GuestFunctionDefinitionBuilder<'a, 'b> {
    #[inline]
    pub fn add_function_name(&mut self, function_name: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            GuestFunctionDefinition::VT_FUNCTION_NAME,
            function_name,
        );
    }
    #[inline]
    pub fn add_parameters(
        &mut self,
        parameters: flatbuffers::WIPOffset<flatbuffers::Vector<'b, ParameterType>>,
    ) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            GuestFunctionDefinition::VT_PARAMETERS,
            parameters,
        );
    }
    #[inline]
    pub fn add_return_type(&mut self, return_type: ReturnType) {
        self.fbb_.push_slot::<ReturnType>(
            GuestFunctionDefinition::VT_RETURN_TYPE,
            return_type,
            ReturnType::hlint,
        );
    }
    #[inline]
    pub fn add_function_pointer(&mut self, function_pointer: i64) {
        self.fbb_.push_slot::<i64>(
            GuestFunctionDefinition::VT_FUNCTION_POINTER,
            function_pointer,
            0,
        );
    }
    #[inline]
    pub fn new(
        _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    ) -> GuestFunctionDefinitionBuilder<'a, 'b> {
        let start = _fbb.start_table();
        GuestFunctionDefinitionBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<GuestFunctionDefinition<'a>> {
        let o = self.fbb_.end_table(self.start_);
        self.fbb_.required(
            o,
            GuestFunctionDefinition::VT_FUNCTION_NAME,
            "function_name",
        );
        self.fbb_
            .required(o, GuestFunctionDefinition::VT_PARAMETERS, "parameters");
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl core::fmt::Debug for GuestFunctionDefinition<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("GuestFunctionDefinition");
        ds.field("function_name", &self.function_name());
        ds.field("parameters", &self.parameters());
        ds.field("return_type", &self.return_type());
        ds.field("function_pointer", &self.function_pointer());
        ds.finish()
    }
}
