// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace Hyperlight.Generated
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct FunctionCall : IFlatbufferObject
{
  private Table __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public static void ValidateVersion() { FlatBufferConstants.FLATBUFFERS_23_3_3(); }
  public static FunctionCall GetRootAsFunctionCall(ByteBuffer _bb) { return GetRootAsFunctionCall(_bb, new FunctionCall()); }
  public static FunctionCall GetRootAsFunctionCall(ByteBuffer _bb, FunctionCall obj) { return (obj.__assign(_bb.GetInt(_bb.Position) + _bb.Position, _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __p = new Table(_i, _bb); }
  public FunctionCall __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public string FunctionName { get { int o = __p.__offset(4); return o != 0 ? __p.__string(o + __p.bb_pos) : null; } }
#if ENABLE_SPAN_T
  public Span<byte> GetFunctionNameBytes() { return __p.__vector_as_span<byte>(4, 1); }
#else
  public ArraySegment<byte>? GetFunctionNameBytes() { return __p.__vector_as_arraysegment(4); }
#endif
  public byte[] GetFunctionNameArray() { return __p.__vector_as_array<byte>(4); }
  public Hyperlight.Generated.Parameter? Parameters(int j) { int o = __p.__offset(6); return o != 0 ? (Hyperlight.Generated.Parameter?)(new Hyperlight.Generated.Parameter()).__assign(__p.__indirect(__p.__vector(o) + j * 4), __p.bb) : null; }
  public int ParametersLength { get { int o = __p.__offset(6); return o != 0 ? __p.__vector_len(o) : 0; } }
  public Hyperlight.Generated.FunctionCallType FunctionCallType { get { int o = __p.__offset(8); return o != 0 ? (Hyperlight.Generated.FunctionCallType)__p.bb.Get(o + __p.bb_pos) : Hyperlight.Generated.FunctionCallType.none; } }
  public Hyperlight.Generated.ReturnType ExpectedReturnType { get { int o = __p.__offset(10); return o != 0 ? (Hyperlight.Generated.ReturnType)__p.bb.Get(o + __p.bb_pos) : Hyperlight.Generated.ReturnType.hlint; } }

  public static Offset<Hyperlight.Generated.FunctionCall> CreateFunctionCall(FlatBufferBuilder builder,
      StringOffset function_nameOffset = default(StringOffset),
      VectorOffset parametersOffset = default(VectorOffset),
      Hyperlight.Generated.FunctionCallType function_call_type = Hyperlight.Generated.FunctionCallType.none,
      Hyperlight.Generated.ReturnType expected_return_type = Hyperlight.Generated.ReturnType.hlint) {
    builder.StartTable(4);
    FunctionCall.AddParameters(builder, parametersOffset);
    FunctionCall.AddFunctionName(builder, function_nameOffset);
    FunctionCall.AddExpectedReturnType(builder, expected_return_type);
    FunctionCall.AddFunctionCallType(builder, function_call_type);
    return FunctionCall.EndFunctionCall(builder);
  }

  public static void StartFunctionCall(FlatBufferBuilder builder) { builder.StartTable(4); }
  public static void AddFunctionName(FlatBufferBuilder builder, StringOffset functionNameOffset) { builder.AddOffset(0, functionNameOffset.Value, 0); }
  public static void AddParameters(FlatBufferBuilder builder, VectorOffset parametersOffset) { builder.AddOffset(1, parametersOffset.Value, 0); }
  public static VectorOffset CreateParametersVector(FlatBufferBuilder builder, Offset<Hyperlight.Generated.Parameter>[] data) { builder.StartVector(4, data.Length, 4); for (int i = data.Length - 1; i >= 0; i--) builder.AddOffset(data[i].Value); return builder.EndVector(); }
  public static VectorOffset CreateParametersVectorBlock(FlatBufferBuilder builder, Offset<Hyperlight.Generated.Parameter>[] data) { builder.StartVector(4, data.Length, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateParametersVectorBlock(FlatBufferBuilder builder, ArraySegment<Offset<Hyperlight.Generated.Parameter>> data) { builder.StartVector(4, data.Count, 4); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateParametersVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<Offset<Hyperlight.Generated.Parameter>>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartParametersVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(4, numElems, 4); }
  public static void AddFunctionCallType(FlatBufferBuilder builder, Hyperlight.Generated.FunctionCallType functionCallType) { builder.AddByte(2, (byte)functionCallType, 0); }
  public static void AddExpectedReturnType(FlatBufferBuilder builder, Hyperlight.Generated.ReturnType expectedReturnType) { builder.AddByte(3, (byte)expectedReturnType, 0); }
  public static Offset<Hyperlight.Generated.FunctionCall> EndFunctionCall(FlatBufferBuilder builder) {
    int o = builder.EndTable();
    builder.Required(o, 4);  // function_name
    return new Offset<Hyperlight.Generated.FunctionCall>(o);
  }
  public static void FinishFunctionCallBuffer(FlatBufferBuilder builder, Offset<Hyperlight.Generated.FunctionCall> offset) { builder.Finish(offset.Value); }
  public static void FinishSizePrefixedFunctionCallBuffer(FlatBufferBuilder builder, Offset<Hyperlight.Generated.FunctionCall> offset) { builder.FinishSizePrefixed(offset.Value); }

  public static VectorOffset CreateSortedVectorOfFunctionCall(FlatBufferBuilder builder, Offset<FunctionCall>[] offsets) {
    Array.Sort(offsets,
      (Offset<FunctionCall> o1, Offset<FunctionCall> o2) =>
        new FunctionCall().__assign(builder.DataBuffer.Length - o1.Value, builder.DataBuffer).FunctionName.CompareTo(new FunctionCall().__assign(builder.DataBuffer.Length - o2.Value, builder.DataBuffer).FunctionName));
    return builder.CreateVectorOfTables(offsets);
  }

  public static FunctionCall? __lookup_by_key(int vectorLocation, string key, ByteBuffer bb) {
    FunctionCall obj_ = new FunctionCall();
    int span = bb.GetInt(vectorLocation - 4);
    int start = 0;
    while (span != 0) {
      int middle = span / 2;
      int tableOffset = Table.__indirect(vectorLocation + 4 * (start + middle), bb);
      obj_.__assign(tableOffset, bb);
      int comp = obj_.FunctionName.CompareTo(key);
      if (comp > 0) {
        span = middle;
      } else if (comp < 0) {
        middle++;
        start += middle;
        span -= middle;
      } else {
        return obj_;
      }
    }
    return null;
  }
}


}