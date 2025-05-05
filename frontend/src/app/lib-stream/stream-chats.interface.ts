import { StringDateTime } from "../common/string-date-time";

export interface ChatMsg {
    id: number;
    date: StringDateTime;
    member: string;
    msg: string;
};

export class ChatMsgUtil {
    public static create(obj: { id: number, date: StringDateTime, member: string, msg: string }): ChatMsg {
        const { id, date, member, msg } = obj;
        return { id, date, member, msg };
    }
    public static getChatMsg(id: number, date: StringDateTime, member: string, msg: string): ChatMsg {
        return { id, date, member, msg };
    }
}

