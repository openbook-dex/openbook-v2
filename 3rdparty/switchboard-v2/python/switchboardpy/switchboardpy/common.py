import io
import six
import anchorpy
from dataclasses import dataclass
from functools import reduce
from typing import Any, Type
from decimal import Decimal
from solana.publickey import PublicKey
from solana.keypair import Keypair
from google.protobuf.internal import decoder
from switchboardpy.compiled import OracleJob


# Devnet Program ID.
SBV2_DEVNET_PID = PublicKey(
    '2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG'
)

# Mainnet-Beta Program ID.
SBV2_MAINNET_PID = PublicKey(
    'SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f'
)

# https://stackoverflow.com/a/34539706
def readRawVarint32(stream: io.BytesIO):
    mask = 0x80 # (1 << 7)
    raw_varint32 = []
    while 1:
        b = stream.read(1)
        #eof
        if b == "":
            break
        raw_varint32.append(b)
        if not (ord(b) & mask):
            #we found a byte starting with a 0, which means it's the last byte of this varint
            break
    return raw_varint32

def getSize(raw_varint32):
    result = 0
    shift = 0
    b = six.indexbytes(raw_varint32, 0)
    result |= ((ord(b) & 0x7f) << shift)
    return result

def readDelimitedFrom(MessageType: Type[OracleJob], stream: io.BytesIO):
    raw_varint32 = readRawVarint32(stream)
    message = None
    if raw_varint32:
        size = getSize(raw_varint32)

        data = stream.read(size)
        if len(data) < size:
            raise Exception("Unexpected end of file")

        message = MessageType()
        message.ParseFromString(data)

    return message

# take OracleJob data and return bytes
def parseOracleJob(data: bytes):
    dataStream = io.BytesIO(data)
    return readDelimitedFrom(OracleJob, dataStream);



# Input parameters for constructing wrapped representations of Switchboard accounts. 
@dataclass
class AccountParams:

    """program referencing the Switchboard program and IDL."""
    program: anchorpy.Program

    """
    Public key of the account being referenced. This will always be populated
    within the account wrapper.
    """
    public_key: PublicKey = None

    """Keypair of the account being referenced. This may not always be populated."""
    keypair: Keypair = None

@dataclass
class SwitchboardDecimal:
    mantissa: int
    scale: int

    """
    Convert BN.js style num and return SwitchboardDecimal

    Args:
        obj (Any): Object with integer fields scale and mantissa (hex val)

    Returns:
        sbd (SwitchboardDecimal): SwitchboardDecimal
    """
    @staticmethod
    def fromObj(obj: Any):
        return SwitchboardDecimal(
            mantissa=obj.mantissa,
            scale=obj.scale
        )
    
    def to_decimal(self, sbd: object):
        mantissa = Decimal(sbd.mantissa)
        scale = sbd.scale
        return mantissa / Decimal(10 ** scale)

    @staticmethod
    def from_decimal(dec: Decimal):
        _, digits, exponent = dec.as_tuple()
        integer = reduce(lambda rst, x: rst * 10 + x, digits)
        return SwitchboardDecimal(integer, exponent)

    # convert any switchboard-decimal-like object to a decimal
    @staticmethod
    def sbd_to_decimal(sbd: object) -> Decimal:
        mantissa = Decimal(sbd.mantissa)
        scale = sbd.scale
        return mantissa / Decimal(10 ** scale)

    # for sending as argument in transaction
    def as_proper_sbd(self, program: anchorpy.Program):
        return program.type['SwitchboardDecimal'](self.mantissa, self.scale)

    def __eq__(self, __o: object) -> bool:
        if not (hasattr(__o, 'mantissa') and hasattr(__o, 'scale')):
            return False
        return self.mantissa == __o.mantissa and self.scale == __o.scale
