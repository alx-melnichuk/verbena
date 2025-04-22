import { StringDateTime } from "../common/string-date-time";

export interface ChatMsg {
    date: StringDateTime;
    member: string;
    msg: string;
};

export class ChatMsgUtil {
    public static create(obj: { date: StringDateTime, member: string, msg: string }): ChatMsg {
        const { date, member, msg } = obj;
        return { date, member, msg };
    }
    public static getChatMsg(date: StringDateTime, member: string, msg: string): ChatMsg {
        return { date, member, msg };
    }
}

