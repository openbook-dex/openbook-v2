import { SwitchboardProgram } from './program';

/**
 * Switchboard wrapper for anchor program errors.
 */
export class SwitchboardError {
  /**
   * Converts a numerical error code to a SwitchboardError based on the program
   * IDL.
   * @param program the Switchboard program object containing the program IDL.
   * @param code Error code to convert to a SwitchboardError object.
   * @return SwitchboardError
   */
  static fromCode(program: SwitchboardProgram, code: number): SwitchboardError {
    for (const e of program.idl.errors ?? []) {
      if (code === e.code) {
        return new SwitchboardError(program, e.name, e.code, e.msg);
      }
    }
    throw new Error(`Could not find SwitchboardError for error code ${code}`);
  }

  /**
   *  The program containing the Switchboard IDL specifying error codes.
   */
  readonly program: SwitchboardProgram;

  /**
   *  Stringified name of the error type.
   */
  readonly name: string;

  /**
   *  Numerical SwitchboardError representation.
   */
  readonly code: number;

  /**
   *  Message describing this error in detail.
   */
  readonly msg?: string;

  private constructor(
    program: SwitchboardProgram,
    name: string,
    code: number,
    msg?: string
  ) {
    this.program = program;
    this.name = name;
    this.code = code;
    this.msg = msg;
  }
}
