type ActionFn = (args: string[], opts?: Record<string, unknown>) => void | Promise<void>;

interface CmdDef {
  description: string;
  action: ActionFn;
  argCount: number;
}

const commands: Record<string, CmdDef> = {};

function cmd(name: string, desc: string, argCount = 0): void {
  const def: CmdDef = {
    description: desc,
    action: (_a: string[], _o?: Record<string, unknown>) => {},
    argCount,
  };
  commands[name] = def;
}

export { cmd, commands, type CmdDef, type ActionFn };
