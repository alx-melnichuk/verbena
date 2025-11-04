import { HttpClient, HttpErrorResponse, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { lastValueFrom } from 'rxjs';

import { Uri } from 'src/app/common/uri';
import { HttpParamsUtil } from '../utils/http-params.util';
import {
    BlockedUserDto, BlockedUserDtoUtil, BlockedUserMiniDto, BlockedUserMiniDtoUtil, ChatMessageDto, SearchChatMessageDto
} from './chat-message-api.interface';

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

    public getBlockedUsersNames(): Promise<string[] | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://blocked_users/nicknames`);
        return lastValueFrom(this.http.get<string[] | HttpErrorResponse>(url));
    }

    public getBlockedUsers(): Promise<BlockedUserDto[] | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.get<BlockedUserDto[] | HttpErrorResponse>(url))
            .then((response) => ((response || []) as BlockedUserDto[]).map((v) => BlockedUserDtoUtil.create(v)));
    }

    public postBlockedUser(blockedNickname: string): Promise<BlockedUserMiniDto | HttpErrorResponse | undefined> {
        if (!blockedNickname) {
            return Promise.resolve(undefined);
        }
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.post<BlockedUserMiniDto | HttpErrorResponse>(url, { blockedNickname }))
            .then((response) => BlockedUserMiniDtoUtil.create(response as BlockedUserMiniDto));
    }

    public deleteBlockedUser(blockedNickname: string): Promise<BlockedUserMiniDto | HttpErrorResponse | undefined> {
        if (!blockedNickname) {
            return Promise.resolve(undefined);
        }
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.delete<BlockedUserMiniDto | HttpErrorResponse>(url, { body: { blockedNickname } }))
            .then((response) => BlockedUserMiniDtoUtil.create(response as BlockedUserMiniDto));
    }
}
