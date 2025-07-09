import { StringDateTime } from '../common/string-date-time';

// ** ChatMessageDto **

export interface ChatMessageDto {
    id: number;
    date: StringDateTime;
    member: string;
    msg: string;
    dateEdt?: StringDateTime | undefined;
    dateRmv?: StringDateTime | undefined;
}

export class ChatMessageDtoUtil {
    public static create(obj: Partial<ChatMessageDto>): ChatMessageDto {
        const result: ChatMessageDto = {
            id: obj.id || 0,
            date: obj.date || '',
            member: obj.member || '',
            msg: obj.msg || '',
        };
        if (obj.dateEdt != null) {
            result['dateEdt'] = obj.dateEdt;
        }
        if (obj.dateRmv != null) {
            result['dateRmv'] = obj.dateRmv;
        }
        return result;
    }
}

// ** SearchChatMessageDto **

export interface SearchChatMessageDto {
    streamId: number;
    isSortDes?: boolean;
    minDate?: StringDateTime;
    maxDate?: StringDateTime;
    limit?: number;
}

// ** ParamQueryPastMsg **

export interface ParamQueryPastMsg {
    isSortDes: boolean;
    minDate?: StringDateTime;
    maxDate?: StringDateTime;
    limit?: number;
}

// ** BlockedUserDto **

export interface BlockedUserDto {
    id: number;
    userId: number;
    blockedId: number;
    blockedNickname: string;
    blockDate: StringDateTime;
}

export class BlockedUserDtoUtil {
    public static create(obj: Partial<BlockedUserDto>): BlockedUserDto {
        let id: number = obj.id || 0;
        let userId: number = obj.userId || 0;
        let blockedId: number = obj.blockedId || 0;
        let blockedNickname: string = obj.blockedNickname || '';
        let blockDate: StringDateTime = obj.blockDate || '';
        return { id, userId, blockedId, blockedNickname, blockDate };
    }
}

// ** - **