import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { lastValueFrom } from 'rxjs';

import { Uri } from 'src/app/common/uri';
import { HttpParamsUtil } from '../utils/http-params.util';
import { BlockedUserDto, BlockedUserDtoUtil, ChatMessageDto, SearchChatMessageDto } from './chat-message-api.interface';

@Injectable({
    providedIn: 'root'
})
export class ChatMessageApiService {

    constructor(private http: HttpClient) { }

    public getChatMessages(searchChatMsgDto: SearchChatMessageDto): Promise<ChatMessageDto[] | HttpErrorResponse | undefined> {
        const params: HttpParams = HttpParamsUtil.create(searchChatMsgDto);
        const url = Uri.appUri('appApi://chat_messages');
        return lastValueFrom(this.http.get<ChatMessageDto[] | HttpErrorResponse>(url, { params }));
    }

    public getBlockedUsers(): Promise<BlockedUserDto[] | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.get<BlockedUserDto[] | HttpErrorResponse>(url))
            .then((response) => ((response || []) as BlockedUserDto[]).map((v) => BlockedUserDtoUtil.create(v)));
    }

    public postBlockedUser(blockedNickname: string): Promise<BlockedUserDto | HttpErrorResponse | undefined> {
        if (!blockedNickname) {
            return Promise.resolve(undefined);
        }
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.post<BlockedUserDto | HttpErrorResponse>(url, { blockedNickname }))
            .then((response) => BlockedUserDtoUtil.create(response as BlockedUserDto));
    }

    public deleteBlockedUser(blockedNickname: string): Promise<BlockedUserDto | HttpErrorResponse | undefined> {
        if (!blockedNickname) {
            return Promise.resolve(undefined);
        }
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.delete<BlockedUserDto | HttpErrorResponse>(url, { body: { blockedNickname } }))
            .then((response) => BlockedUserDtoUtil.create(response as BlockedUserDto));
    }
}
