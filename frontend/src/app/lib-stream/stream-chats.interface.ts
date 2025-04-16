import { StringDateTime } from "../common/string-date-time";

export interface ChatMsg {
    date: StringDateTime;
    member: string;
    msg: string;
};

export interface RefreshChatMsgs {
    refreshAddChatMsg(chatMsg: ChatMsg): void;
    refreshEditChatMsg(chatMsg: ChatMsg): void;
    refreshRemoveChatMsg(msgDate: StringDateTime): void;
};

