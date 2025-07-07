import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { ChatMessageApiService } from './chat-message-api.service';
import { BlockedUserDto, ChatMessageDto } from './chat-message-api.interface';
import { StringDateTime } from '../common/string-date-time';

@Injectable({
    providedIn: 'root'
})
export class ChatMessageService {

    constructor(private chatMessageApiService: ChatMessageApiService) { }

    public getChatMessages(
        streamId: number, isSortDes?: boolean, minDate?: StringDateTime, limit?: number
    ): Promise<ChatMessageDto[] | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.getChatMessages({ streamId, isSortDes, minDate/*, limit: 10*/ }) // TODO del "limit: 10"
    }

    public getBlockedUsers(stream_id: number): Promise<BlockedUserDto[] | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.getBlockedUsers(stream_id);
    }
}
