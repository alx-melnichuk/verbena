import { StringDateTime } from '../common/string-date-time';
import { StringDateTimeUtil } from '../utils/string-date-time.util';

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

// ** BlockedUserMiniDto **

export interface BlockedUserMiniDto {
    id: number;
    userId: number;
    nickname: string;
    blockDate: Date | null;
}

export class BlockedUserMiniDtoUtil {
    public static create(obj: Partial<BlockedUserMiniDto> | null | undefined): BlockedUserMiniDto {
        let id: number = obj?.id || 0;
        let userId: number = obj?.userId || 0;
        let nickname: string = obj?.nickname || '';
        let blockDateStr = obj?.blockDate || '';
        const isString = blockDateStr != null && typeof blockDateStr == 'string';
        let blockDate = isString ? StringDateTimeUtil.toDate(blockDateStr) : null;
        return { id, userId, nickname, blockDate };
    }
}

// ** BlockedUserDto **

export interface BlockedUserDto {
    id: number;
    userId: number;
    nickname: string;
    email: string;
    blockDate: Date | null;
    avatar: string;
}

export class BlockedUserDtoUtil {
    public static create(obj: Partial<BlockedUserDto> | null | undefined): BlockedUserDto {
        let id: number = obj?.id || 0;
        let userId: number = obj?.userId || 0;
        let nickname: string = obj?.nickname || '';
        let email: string = obj?.email || '';
        let blockDateStr = obj?.blockDate || '';
        const isString = blockDateStr != null && typeof blockDateStr == 'string';
        let blockDate = isString ? StringDateTimeUtil.toDate(blockDateStr) : null;
        let avatar: string = obj?.avatar || '';
        return { id, userId, nickname, email, blockDate, avatar };
    }
}

// ** - **