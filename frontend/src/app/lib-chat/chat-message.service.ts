import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { ChatMessageApiService } from './chat-message-api.service';
import { BlockedUserDto, BlockedUserMiniDto, ChatMessageDto } from './chat-message-api.interface';
import { StringDateTime } from '../common/string-date-time';

@Injectable({
    providedIn: 'root'
})
export class ChatMessageService {

    constructor(private chatMessageApiService: ChatMessageApiService) { }

    public getChatMessages(
        streamId: number, isSortDes?: boolean, minDate?: StringDateTime, maxDate?: StringDateTime, limit?: number
    ): Promise<ChatMessageDto[] | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.getChatMessages({ streamId, isSortDes, minDate, maxDate })
    }

    public getBlockedUsersNames(): Promise<string[] | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.getBlockedUsersNames();
    }

    public getBlockedUsers(sortColumn?: string, sortDesc?: boolean): Promise<BlockedUserDto[] | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.getBlockedUsers({ sortColumn, sortDesc });
    }

    public postBlockedUser(blockedNickname: string): Promise<BlockedUserMiniDto | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.postBlockedUser(blockedNickname);
    }

    public deleteBlockedUser(blockedNickname: string): Promise<BlockedUserMiniDto | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.deleteBlockedUser(blockedNickname);
    }
}
