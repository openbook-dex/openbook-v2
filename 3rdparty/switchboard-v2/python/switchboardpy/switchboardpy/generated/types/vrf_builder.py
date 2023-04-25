from __future__ import annotations
from . import (
    ecvrf_proof_zc,
    scalar,
    field_element_zc,
    completed_point_zc,
    edwards_point_zc,
    ecvrf_intermediate,
    vrf_status,
)
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class VrfBuilderJSON(typing.TypedDict):
    producer: str
    status: vrf_status.VrfStatusJSON
    repr_proof: list[int]
    proof: ecvrf_proof_zc.EcvrfProofZCJSON
    y_point: str
    stage: int
    stage1_out: ecvrf_intermediate.EcvrfIntermediateJSON
    r1: edwards_point_zc.EdwardsPointZCJSON
    r2: edwards_point_zc.EdwardsPointZCJSON
    stage3_out: ecvrf_intermediate.EcvrfIntermediateJSON
    h_point: edwards_point_zc.EdwardsPointZCJSON
    s_reduced: scalar.ScalarJSON
    y_point_builder: list[field_element_zc.FieldElementZCJSON]
    y_ristretto_point: edwards_point_zc.EdwardsPointZCJSON
    mul_round: int
    hash_points_round: int
    mul_tmp1: completed_point_zc.CompletedPointZCJSON
    u_point1: edwards_point_zc.EdwardsPointZCJSON
    u_point2: edwards_point_zc.EdwardsPointZCJSON
    v_point1: edwards_point_zc.EdwardsPointZCJSON
    v_point2: edwards_point_zc.EdwardsPointZCJSON
    u_point: edwards_point_zc.EdwardsPointZCJSON
    v_point: edwards_point_zc.EdwardsPointZCJSON
    u1: field_element_zc.FieldElementZCJSON
    u2: field_element_zc.FieldElementZCJSON
    invertee: field_element_zc.FieldElementZCJSON
    y: field_element_zc.FieldElementZCJSON
    z: field_element_zc.FieldElementZCJSON
    p1_bytes: list[int]
    p2_bytes: list[int]
    p3_bytes: list[int]
    p4_bytes: list[int]
    c_prime_hashbuf: list[int]
    m1: field_element_zc.FieldElementZCJSON
    m2: field_element_zc.FieldElementZCJSON
    tx_remaining: int
    verified: bool
    result: list[int]


@dataclass
class VrfBuilder:
    layout: typing.ClassVar = borsh.CStruct(
        "producer" / BorshPubkey,
        "status" / vrf_status.layout,
        "repr_proof" / borsh.U8[80],
        "proof" / ecvrf_proof_zc.EcvrfProofZC.layout,
        "y_point" / BorshPubkey,
        "stage" / borsh.U32,
        "stage1_out" / ecvrf_intermediate.EcvrfIntermediate.layout,
        "r1" / edwards_point_zc.EdwardsPointZC.layout,
        "r2" / edwards_point_zc.EdwardsPointZC.layout,
        "stage3_out" / ecvrf_intermediate.EcvrfIntermediate.layout,
        "h_point" / edwards_point_zc.EdwardsPointZC.layout,
        "s_reduced" / scalar.Scalar.layout,
        "y_point_builder" / field_element_zc.FieldElementZC.layout[3],
        "y_ristretto_point" / edwards_point_zc.EdwardsPointZC.layout,
        "mul_round" / borsh.U8,
        "hash_points_round" / borsh.U8,
        "mul_tmp1" / completed_point_zc.CompletedPointZC.layout,
        "u_point1" / edwards_point_zc.EdwardsPointZC.layout,
        "u_point2" / edwards_point_zc.EdwardsPointZC.layout,
        "v_point1" / edwards_point_zc.EdwardsPointZC.layout,
        "v_point2" / edwards_point_zc.EdwardsPointZC.layout,
        "u_point" / edwards_point_zc.EdwardsPointZC.layout,
        "v_point" / edwards_point_zc.EdwardsPointZC.layout,
        "u1" / field_element_zc.FieldElementZC.layout,
        "u2" / field_element_zc.FieldElementZC.layout,
        "invertee" / field_element_zc.FieldElementZC.layout,
        "y" / field_element_zc.FieldElementZC.layout,
        "z" / field_element_zc.FieldElementZC.layout,
        "p1_bytes" / borsh.U8[32],
        "p2_bytes" / borsh.U8[32],
        "p3_bytes" / borsh.U8[32],
        "p4_bytes" / borsh.U8[32],
        "c_prime_hashbuf" / borsh.U8[16],
        "m1" / field_element_zc.FieldElementZC.layout,
        "m2" / field_element_zc.FieldElementZC.layout,
        "tx_remaining" / borsh.U32,
        "verified" / borsh.Bool,
        "result" / borsh.U8[32],
    )
    producer: PublicKey
    status: vrf_status.VrfStatusKind
    repr_proof: list[int]
    proof: ecvrf_proof_zc.EcvrfProofZC
    y_point: PublicKey
    stage: int
    stage1_out: ecvrf_intermediate.EcvrfIntermediate
    r1: edwards_point_zc.EdwardsPointZC
    r2: edwards_point_zc.EdwardsPointZC
    stage3_out: ecvrf_intermediate.EcvrfIntermediate
    h_point: edwards_point_zc.EdwardsPointZC
    s_reduced: scalar.Scalar
    y_point_builder: list[field_element_zc.FieldElementZC]
    y_ristretto_point: edwards_point_zc.EdwardsPointZC
    mul_round: int
    hash_points_round: int
    mul_tmp1: completed_point_zc.CompletedPointZC
    u_point1: edwards_point_zc.EdwardsPointZC
    u_point2: edwards_point_zc.EdwardsPointZC
    v_point1: edwards_point_zc.EdwardsPointZC
    v_point2: edwards_point_zc.EdwardsPointZC
    u_point: edwards_point_zc.EdwardsPointZC
    v_point: edwards_point_zc.EdwardsPointZC
    u1: field_element_zc.FieldElementZC
    u2: field_element_zc.FieldElementZC
    invertee: field_element_zc.FieldElementZC
    y: field_element_zc.FieldElementZC
    z: field_element_zc.FieldElementZC
    p1_bytes: list[int]
    p2_bytes: list[int]
    p3_bytes: list[int]
    p4_bytes: list[int]
    c_prime_hashbuf: list[int]
    m1: field_element_zc.FieldElementZC
    m2: field_element_zc.FieldElementZC
    tx_remaining: int
    verified: bool
    result: list[int]

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfBuilder":
        return cls(
            producer=obj.producer,
            status=vrf_status.from_decoded(obj.status),
            repr_proof=obj.repr_proof,
            proof=ecvrf_proof_zc.EcvrfProofZC.from_decoded(obj.proof),
            y_point=obj.y_point,
            stage=obj.stage,
            stage1_out=ecvrf_intermediate.EcvrfIntermediate.from_decoded(
                obj.stage1_out
            ),
            r1=edwards_point_zc.EdwardsPointZC.from_decoded(obj.r1),
            r2=edwards_point_zc.EdwardsPointZC.from_decoded(obj.r2),
            stage3_out=ecvrf_intermediate.EcvrfIntermediate.from_decoded(
                obj.stage3_out
            ),
            h_point=edwards_point_zc.EdwardsPointZC.from_decoded(obj.h_point),
            s_reduced=scalar.Scalar.from_decoded(obj.s_reduced),
            y_point_builder=list(
                map(
                    lambda item: field_element_zc.FieldElementZC.from_decoded(item),
                    obj.y_point_builder,
                )
            ),
            y_ristretto_point=edwards_point_zc.EdwardsPointZC.from_decoded(
                obj.y_ristretto_point
            ),
            mul_round=obj.mul_round,
            hash_points_round=obj.hash_points_round,
            mul_tmp1=completed_point_zc.CompletedPointZC.from_decoded(obj.mul_tmp1),
            u_point1=edwards_point_zc.EdwardsPointZC.from_decoded(obj.u_point1),
            u_point2=edwards_point_zc.EdwardsPointZC.from_decoded(obj.u_point2),
            v_point1=edwards_point_zc.EdwardsPointZC.from_decoded(obj.v_point1),
            v_point2=edwards_point_zc.EdwardsPointZC.from_decoded(obj.v_point2),
            u_point=edwards_point_zc.EdwardsPointZC.from_decoded(obj.u_point),
            v_point=edwards_point_zc.EdwardsPointZC.from_decoded(obj.v_point),
            u1=field_element_zc.FieldElementZC.from_decoded(obj.u1),
            u2=field_element_zc.FieldElementZC.from_decoded(obj.u2),
            invertee=field_element_zc.FieldElementZC.from_decoded(obj.invertee),
            y=field_element_zc.FieldElementZC.from_decoded(obj.y),
            z=field_element_zc.FieldElementZC.from_decoded(obj.z),
            p1_bytes=obj.p1_bytes,
            p2_bytes=obj.p2_bytes,
            p3_bytes=obj.p3_bytes,
            p4_bytes=obj.p4_bytes,
            c_prime_hashbuf=obj.c_prime_hashbuf,
            m1=field_element_zc.FieldElementZC.from_decoded(obj.m1),
            m2=field_element_zc.FieldElementZC.from_decoded(obj.m2),
            tx_remaining=obj.tx_remaining,
            verified=obj.verified,
            result=obj.result,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "producer": self.producer,
            "status": self.status.to_encodable(),
            "repr_proof": self.repr_proof,
            "proof": self.proof.to_encodable(),
            "y_point": self.y_point,
            "stage": self.stage,
            "stage1_out": self.stage1_out.to_encodable(),
            "r1": self.r1.to_encodable(),
            "r2": self.r2.to_encodable(),
            "stage3_out": self.stage3_out.to_encodable(),
            "h_point": self.h_point.to_encodable(),
            "s_reduced": self.s_reduced.to_encodable(),
            "y_point_builder": list(
                map(lambda item: item.to_encodable(), self.y_point_builder)
            ),
            "y_ristretto_point": self.y_ristretto_point.to_encodable(),
            "mul_round": self.mul_round,
            "hash_points_round": self.hash_points_round,
            "mul_tmp1": self.mul_tmp1.to_encodable(),
            "u_point1": self.u_point1.to_encodable(),
            "u_point2": self.u_point2.to_encodable(),
            "v_point1": self.v_point1.to_encodable(),
            "v_point2": self.v_point2.to_encodable(),
            "u_point": self.u_point.to_encodable(),
            "v_point": self.v_point.to_encodable(),
            "u1": self.u1.to_encodable(),
            "u2": self.u2.to_encodable(),
            "invertee": self.invertee.to_encodable(),
            "y": self.y.to_encodable(),
            "z": self.z.to_encodable(),
            "p1_bytes": self.p1_bytes,
            "p2_bytes": self.p2_bytes,
            "p3_bytes": self.p3_bytes,
            "p4_bytes": self.p4_bytes,
            "c_prime_hashbuf": self.c_prime_hashbuf,
            "m1": self.m1.to_encodable(),
            "m2": self.m2.to_encodable(),
            "tx_remaining": self.tx_remaining,
            "verified": self.verified,
            "result": self.result,
        }

    def to_json(self) -> VrfBuilderJSON:
        return {
            "producer": str(self.producer),
            "status": self.status.to_json(),
            "repr_proof": self.repr_proof,
            "proof": self.proof.to_json(),
            "y_point": str(self.y_point),
            "stage": self.stage,
            "stage1_out": self.stage1_out.to_json(),
            "r1": self.r1.to_json(),
            "r2": self.r2.to_json(),
            "stage3_out": self.stage3_out.to_json(),
            "h_point": self.h_point.to_json(),
            "s_reduced": self.s_reduced.to_json(),
            "y_point_builder": list(
                map(lambda item: item.to_json(), self.y_point_builder)
            ),
            "y_ristretto_point": self.y_ristretto_point.to_json(),
            "mul_round": self.mul_round,
            "hash_points_round": self.hash_points_round,
            "mul_tmp1": self.mul_tmp1.to_json(),
            "u_point1": self.u_point1.to_json(),
            "u_point2": self.u_point2.to_json(),
            "v_point1": self.v_point1.to_json(),
            "v_point2": self.v_point2.to_json(),
            "u_point": self.u_point.to_json(),
            "v_point": self.v_point.to_json(),
            "u1": self.u1.to_json(),
            "u2": self.u2.to_json(),
            "invertee": self.invertee.to_json(),
            "y": self.y.to_json(),
            "z": self.z.to_json(),
            "p1_bytes": self.p1_bytes,
            "p2_bytes": self.p2_bytes,
            "p3_bytes": self.p3_bytes,
            "p4_bytes": self.p4_bytes,
            "c_prime_hashbuf": self.c_prime_hashbuf,
            "m1": self.m1.to_json(),
            "m2": self.m2.to_json(),
            "tx_remaining": self.tx_remaining,
            "verified": self.verified,
            "result": self.result,
        }

    @classmethod
    def from_json(cls, obj: VrfBuilderJSON) -> "VrfBuilder":
        return cls(
            producer=PublicKey(obj["producer"]),
            status=vrf_status.from_json(obj["status"]),
            repr_proof=obj["repr_proof"],
            proof=ecvrf_proof_zc.EcvrfProofZC.from_json(obj["proof"]),
            y_point=PublicKey(obj["y_point"]),
            stage=obj["stage"],
            stage1_out=ecvrf_intermediate.EcvrfIntermediate.from_json(
                obj["stage1_out"]
            ),
            r1=edwards_point_zc.EdwardsPointZC.from_json(obj["r1"]),
            r2=edwards_point_zc.EdwardsPointZC.from_json(obj["r2"]),
            stage3_out=ecvrf_intermediate.EcvrfIntermediate.from_json(
                obj["stage3_out"]
            ),
            h_point=edwards_point_zc.EdwardsPointZC.from_json(obj["h_point"]),
            s_reduced=scalar.Scalar.from_json(obj["s_reduced"]),
            y_point_builder=list(
                map(
                    lambda item: field_element_zc.FieldElementZC.from_json(item),
                    obj["y_point_builder"],
                )
            ),
            y_ristretto_point=edwards_point_zc.EdwardsPointZC.from_json(
                obj["y_ristretto_point"]
            ),
            mul_round=obj["mul_round"],
            hash_points_round=obj["hash_points_round"],
            mul_tmp1=completed_point_zc.CompletedPointZC.from_json(obj["mul_tmp1"]),
            u_point1=edwards_point_zc.EdwardsPointZC.from_json(obj["u_point1"]),
            u_point2=edwards_point_zc.EdwardsPointZC.from_json(obj["u_point2"]),
            v_point1=edwards_point_zc.EdwardsPointZC.from_json(obj["v_point1"]),
            v_point2=edwards_point_zc.EdwardsPointZC.from_json(obj["v_point2"]),
            u_point=edwards_point_zc.EdwardsPointZC.from_json(obj["u_point"]),
            v_point=edwards_point_zc.EdwardsPointZC.from_json(obj["v_point"]),
            u1=field_element_zc.FieldElementZC.from_json(obj["u1"]),
            u2=field_element_zc.FieldElementZC.from_json(obj["u2"]),
            invertee=field_element_zc.FieldElementZC.from_json(obj["invertee"]),
            y=field_element_zc.FieldElementZC.from_json(obj["y"]),
            z=field_element_zc.FieldElementZC.from_json(obj["z"]),
            p1_bytes=obj["p1_bytes"],
            p2_bytes=obj["p2_bytes"],
            p3_bytes=obj["p3_bytes"],
            p4_bytes=obj["p4_bytes"],
            c_prime_hashbuf=obj["c_prime_hashbuf"],
            m1=field_element_zc.FieldElementZC.from_json(obj["m1"]),
            m2=field_element_zc.FieldElementZC.from_json(obj["m2"]),
            tx_remaining=obj["tx_remaining"],
            verified=obj["verified"],
            result=obj["result"],
        )
