import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { lastValueFrom } from 'rxjs';

import { Uri } from 'src/app/common/uri';
import { HttpParamsUtil } from '../utils/http-params.util';
import { ChatMessageDto, FilterChatMessageDto } from './chat-message-api.interface';

@Injectable({
    providedIn: 'root'
})
export class ChatMessageApiService {

    constructor(private http: HttpClient) { }

    public getChatMessages(filterChatMsgDto: FilterChatMessageDto): Promise<ChatMessageDto[] | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create(filterChatMsgDto);
        const url = Uri.appUri('appApi://chat_messages');
        return lastValueFrom(this.http.get<ChatMessageDto[] | HttpErrorResponse>(url, { params }));
    }
}
