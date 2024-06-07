// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace Hyperlight.Generated
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct hluint : IFlatbufferObject
{
  private Table __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public static void ValidateVersion() { FlatBufferConstants.FLATBUFFERS_23_5_26(); }
  public static hluint GetRootAshluint(ByteBuffer _bb) { return GetRootAshluint(_bb, new hluint()); }
  public static hluint GetRootAshluint(ByteBuffer _bb, hluint obj) { return (obj.__assign(_bb.GetInt(_bb.Position) + _bb.Position, _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __p = new Table(_i, _bb); }
  public hluint __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public uint Value { get { int o = __p.__offset(4); return o != 0 ? __p.bb.GetUint(o + __p.bb_pos) : (uint)0; } }

  public static Offset<Hyperlight.Generated.hluint> Createhluint(FlatBufferBuilder builder,
      uint value = 0) {
    builder.StartTable(1);
    hluint.AddValue(builder, value);
    return hluint.Endhluint(builder);
  }

  public static void Starthluint(FlatBufferBuilder builder) { builder.StartTable(1); }
  public static void AddValue(FlatBufferBuilder builder, uint value) { builder.AddUint(0, value, 0); }
  public static Offset<Hyperlight.Generated.hluint> Endhluint(FlatBufferBuilder builder) {
    int o = builder.EndTable();
    return new Offset<Hyperlight.Generated.hluint>(o);
  }
  public hluintT UnPack() {
    var _o = new hluintT();
    this.UnPackTo(_o);
    return _o;
  }
  public void UnPackTo(hluintT _o) {
    _o.Value = this.Value;
  }
  public static Offset<Hyperlight.Generated.hluint> Pack(FlatBufferBuilder builder, hluintT _o) {
    if (_o == null) return default(Offset<Hyperlight.Generated.hluint>);
    return Createhluint(
      builder,
      _o.Value);
  }
}

public class hluintT
{
  public uint Value { get; set; }

  public hluintT() {
    this.Value = 0;
  }
}


static public class hluintVerify
{
  static public bool Verify(Google.FlatBuffers.Verifier verifier, uint tablePos)
  {
    return verifier.VerifyTableStart(tablePos)
      && verifier.VerifyField(tablePos, 4 /*Value*/, 4 /*uint*/, 4, false)
      && verifier.VerifyTableEnd(tablePos);
  }
}

}