import { inject, Injectable } from "@angular/core";
import { HttpClient, HttpErrorResponse, HttpParams } from "@angular/common/http";
import { lastValueFrom } from "rxjs";
import { StringDateTime } from "../common/string-date-time";
import { Uri } from "../common/uri";
import { HttpParamsUtil } from "../utils/http-params.util";
import {
    ChatMessageDto, SearchChatMessageDto, BlockedUserDto, SortingBlockedUsersDto, BlockedUserDtoUtil, BlockedUserMiniDto, BlockedUserMiniDtoUtil
} from "./chat-message";


@Injectable({
    providedIn: "root",
})
export class ChatMessageSrv {
    private http: HttpClient = inject(HttpClient);

    public getChatMessages(
        streamId: number, isSortDes?: boolean, minDate?: StringDateTime, maxDate?: StringDateTime, limit?: number
    ): Promise<ChatMessageDto[] | HttpErrorResponse | undefined> {
        const searchChatMsgDto: SearchChatMessageDto = { streamId, isSortDes, minDate, maxDate, limit };
        const params: HttpParams = HttpParamsUtil.create(searchChatMsgDto);
        const url = Uri.appUri("appApi://chat_messages");
        return lastValueFrom(this.http.get<ChatMessageDto[] | HttpErrorResponse>(url, { params }));
    }

    public getBlockedUsersNames(): Promise<string[] | HttpErrorResponse | undefined> {
        const url = Uri.appUri(`appApi://blocked_users/nicknames`);
        return lastValueFrom(this.http.get<string[] | HttpErrorResponse>(url));
    }

    public getBlockedUsers(sortColumn?: string, sortDesc?: boolean): Promise<BlockedUserDto[] | HttpErrorResponse | undefined> {
        const sortingBlockedUsersDto: SortingBlockedUsersDto = { sortColumn, sortDesc };
        const params: HttpParams = HttpParamsUtil.create(sortingBlockedUsersDto);
        const url = Uri.appUri(`appApi://blocked_users`);
        return lastValueFrom(this.http.get<BlockedUserDto[] | HttpErrorResponse>(url, { params }))
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
