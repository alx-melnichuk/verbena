import { StringDateTime } from '../common/string-date-time';

export interface ChatMessageDto {
    id: number;
    date: StringDateTime;
    member: string;
    msg: string;
    isEdt: boolean,
    isRmv: boolean,
}


export class ChatMessageDtoUtil {
    public static create(obj: Partial<ChatMessageDto>): ChatMessageDto {
        let id: number = obj.id || 0;
        let date: StringDateTime = obj.date || '';
        let member: string = obj.member || '';
        let msg: string = obj.msg || '';
        let isEdt: boolean = obj.isEdt != null ? obj.isEdt : false;
        let isRmv: boolean = obj.isRmv != null ? obj.isRmv : false;

        return { id, date, member, msg, isEdt, isRmv };
    }
    public static getChatMessageDto(
        id: number, date: StringDateTime, member: string, msg: string, isEdt1?: boolean, isRmv1?: boolean
    ): ChatMessageDto {
        let isEdt: boolean = isEdt1 != null ? isEdt1 : false;
        let isRmv: boolean = isRmv1 != null ? isRmv1 : false;

        return { id, date, member, msg, isEdt, isRmv };
    }
}


export interface FilterChatMessageDto {
    streamId: number;
    isSortDes?: boolean;
    borderById?: number;
    limit?: number;
}
