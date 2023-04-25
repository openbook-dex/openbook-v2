import os
from solana.publickey import PublicKey

os.environ.setdefault('SWITCHBOARD_PID', '2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG')


PROGRAM_ID = PublicKey(os.environ['SWITCHBOARD_PID'] or "SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f")
