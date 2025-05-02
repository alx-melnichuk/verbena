import { StringDateTime } from "../common/string-date-time";


export enum EWSType {
    Block = 'block', // -
    Count = 'count',
    Echo = 'echo', // -
    Err = 'err',
    Join = 'join',
    Leave = 'leave',
    Msg = 'msg',
    MsgCut = 'msgCut', // -
    MsgPut = 'msgPut', // -
    Name = 'name',
    Unblock = 'unblock', // -
}

export class EWSTypeUtil {
    public static iterator(): EWSType[] {
        return [
            EWSType.Block,
            EWSType.Count,
            EWSType.Echo,
            EWSType.Err,
            EWSType.Join,
            EWSType.Leave,
            EWSType.Msg,
            EWSType.MsgCut,
            EWSType.MsgPut,
            EWSType.Name,
            EWSType.Unblock,
        ];
    }
    public static parse(data: string): EWSType | null {
        let result: EWSType | null = null;
        let dataStr = data.toLowerCase();
        const list: EWSType[] = this.iterator();
        let idx = 0;
        while (idx < list.length && !result) {
            const elem = list[idx];
            if (dataStr == elem.toLowerCase()) {
                result = elem;
            }
            idx++;
        }
        return result;
    }

    public static getBlockEWS(block: string, count?: number): string {
        return JSON.stringify({ ...{ block }, ...{ count } });
    }
    public static getCountEWS(): string {
        return JSON.stringify({ count: -1 });
    }
    public static getEchoEWS(echo: string): string {
        return JSON.stringify({ echo });
    }
    public static getErrEWS(err: string): string {
        return JSON.stringify({ err });
    }
    public static getJoinEWS(join: number, member?: string, count?: number, access?: string): string {
        return JSON.stringify({ ...{ join }, ...{ member }, ...{ count }, ...{ access } });
    }
    public static getLeaveEWS(member?: string, count?: number): string {
        return JSON.stringify({ ...{ leave: '' }, ...{ member }, ...{ count } });
    }
    public static getMsgEWS(msg: string, member?: string, date?: StringDateTime): string {
        return JSON.stringify({ ...{ msg }, ...{ member }, ...{ date } });
    }
    public static getMsgPutEWS(msgPut: string, member?: string, date?: StringDateTime): string {
        return JSON.stringify({ ...{ msgPut }, ...{ member }, ...{ date } });
    }
    public static getMsgCutEWS(msgCut: string, member?: string, date?: StringDateTime): string {
        return JSON.stringify({ ...{ msgCut }, ...{ member }, ...{ date } });
    }
    public static getNameEWS(name: string): string {
        return JSON.stringify({ name });
    }
    public static getUnblockEWS(unblock: string, count?: number): string {
        return JSON.stringify({ ...{ unblock }, ...{ count } });
    }
}

export class EventWS {
    et: EWSType;
    params: Map<string, string>;

    constructor(et: EWSType, params?: Map<string, string> | null | undefined) {
        this.et = et;
        this.params = (params || new Map());
    }

    // Parse input data of ws event.
    public static parse(event: string): EventWS | null {
        const errStartCorrect = !event.startsWith('{') ? `Serialization: missing \"{\".` : '';
        const errEndCorrect = !event.endsWith('}') ? `Serialization: missing \"}\".` : '';
        if (!!errStartCorrect || !!errEndCorrect) {
            console.error(errStartCorrect || errEndCorrect);
            return null;
        }
        // Get the name of the first tag.
        let buf = event.split("\"");
        // let first_tag = buf.next().unwrap_or("");
        let firstTag = buf.length > 1 ? buf[1] : '';
        let ewsType = EWSTypeUtil.parse(firstTag);
        if (ewsType == null) {
            console.error(`unknown command: ${event}`);
            return null;
        }
        const params: Map<string, string> = new Map();
        // Parse the input data.
        let eventObj = JSON.parse(event);
        const keys = Object.keys(eventObj);
        for (let idx = 0; idx < keys.length; idx++) {
            const key = keys[idx];
            params.set(key, String(eventObj[key]));
        }
        return new EventWS(ewsType, params);
    }

    public getStr(name: string): string | undefined {
        return this.params.get(name);
    }
    public getInt(name: string): number | undefined {
        const res = parseInt(this.params.get(name) || '0', 10);
        return !Number.isNaN(res) ? res : undefined;
    }
}

