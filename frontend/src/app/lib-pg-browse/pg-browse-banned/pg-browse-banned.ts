import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { environment as env } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { LOG_PG_BROWSE } from '../../common/routes';
import { Spinner } from '../../components/spinner/spinner';
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { BlockedUserDto } from "../../lib-chat/chat-message";
import { ChatMessageSrv } from "../../lib-chat/chat-message-srv";
import { HttpErrorUtil } from "../../utils/http-error.util";
import { PanelBannedUsers } from '../panel-banned-users/panel-banned-users';

export const SORT_COL_INIT: string = "nickname";
export const SORT_DESC_INIT: boolean = false;

@Component({
    selector: 'app-pg-browse-banned',
    exportAs: "appPgBrowseBanned",
    standalone: true,
    imports: [CommonModule, Spinner, PanelBannedUsers],
    templateUrl: './pg-browse-banned.html',
    styleUrl: './pg-browse-banned.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgBrowseBanned {
    private alertSrv: AlertSrv = inject(AlertSrv);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private chatMessageSrv: ChatMessageSrv = inject(ChatMessageSrv);
    private dialogSrv: DialogSrv = inject(DialogSrv);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private translateSrv: TranslateService = inject(TranslateService);


    public blockedUsers: BlockedUserDto[] = this.route.snapshot.data["blockedUsers"] || [];
    public isLoading: boolean = false;
    public sortColumn: string = SORT_COL_INIT;
    public sortDesc: boolean = SORT_DESC_INIT;

    constructor() {
        if (env.logLevel & LOG_PG_BROWSE) { console.info(`PgBrowseBanned(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
    }

    // ** Public API **

    public doSort(event: Record<string, boolean>): void {
        const keys = Object.keys(event);
        const sortColumn = keys.length > 0 ? keys[0] : "";
        const sortDesc: boolean | undefined = !!sortColumn ? event[sortColumn] : false;
        if (!sortColumn) {
            return;
        }
        this.isLoading = true;
        this.chatMessageSrv.getBlockedUsers(sortColumn, sortDesc)
            .then((response: BlockedUserDto[] | HttpErrorResponse | undefined) => {
                this.blockedUsers = (response as BlockedUserDto[]); // List of past chat messages.
                this.sortColumn = sortColumn;
                this.sortDesc = sortDesc;
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-browse-banned.error_get_blocked_users");
                throw err;
            })
            .finally(() => {
                this.isLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    public doUnblockUser(nickname: string, blockedUsers: BlockedUserDto[]): void {
        const idx = !!nickname ? blockedUsers.findIndex((v) => v.nickname == nickname) : -1;
        if (!nickname || idx == -1) {
            return;
        }

        const message = this.translateSrv.instant("pg-browse-banned.are_you_want_to_unblock_user", { nickname });
        this.dialogSrv.openConfirmation(message, "", { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" })
            .then((res) => {
                if (!!res) {
                    this.deleteBlockedUser(nickname);
                }
            });
    }

    // ** Private API **

    private deleteBlockedUser(nickname: string) {
        if (!nickname) {
            return;
        }
        this.isLoading = true;
        this.chatMessageSrv.deleteBlockedUser(nickname)
            .then(() => {
                Promise.resolve().then(() =>
                    this.doSort({ [this.sortColumn]: this.sortDesc }));
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-browse-banned.error_get_blocked_users");
                throw err;
            })
            .finally(() => {
                this.isLoading = false;
                this.changeDetector.markForCheck();
            });
    }
}
