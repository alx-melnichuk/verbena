import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { ChatMessageApiService } from './chat-message-api.service';
import { ChatMessageDto } from './chat-message-api.interface';

@Injectable({
    providedIn: 'root'
})
export class ChatMessageService {

    constructor(private chatMessageApiService: ChatMessageApiService) { }

    public getChatMessages(
        streamId: number, isSortDes?: boolean, borderById?: number, limit?: number
    ): Promise<ChatMessageDto[] | HttpErrorResponse | undefined> {
        return this.chatMessageApiService.getChatMessages({ streamId, isSortDes, borderById, limit })
    }
}
