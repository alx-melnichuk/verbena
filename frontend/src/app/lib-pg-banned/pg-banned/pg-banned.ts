import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { Component, ViewEncapsulation, ChangeDetectionStrategy, inject, ChangeDetectorRef } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { ActivatedRoute } from "@angular/router";
import { TranslateService, LangChangeEvent } from "@ngx-translate/core";
import { environment } from "../../../environments/environment";
import { LocaleSrv } from "../../common/locale-srv";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { BlockedUserDto } from "../../lib-chat/chat-message";
import { ChatMessageSrv } from "../../lib-chat/chat-message-srv";
import { HttpErrorUtil } from "../../utils/http-error.util";
import { PanelBannedView } from "../panel-banned-view/panel-banned-view";

export const SORT_COL_INIT: string = "nickname";
export const SORT_DESC_INIT: boolean = false;

@Component({
    selector: "app-pg-banned",
    exportAs: "appPgBanned",
    standalone: true,
    imports: [CommonModule, PanelBannedView],
    templateUrl: "./pg-banned.html",
    styleUrl: "./pg-banned.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [ // Create a separate instance of "AlertSrv" to access translations of the current module.
        { provide: AlertSrv, useClass: AlertSrv },
        { provide: DialogSrv, useClass: DialogSrv },
    ],
})
export class PgBanned {
    private alertSrv: AlertSrv = inject(AlertSrv);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private chatMessageSrv: ChatMessageSrv = inject(ChatMessageSrv);
    private localeSrv: LocaleSrv = inject(LocaleSrv);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private translateSrv: TranslateService = inject(TranslateService);

    // The current locale is loaded in the resolver.
    private loadedLangs: Record<string, boolean> = { [this.localeSrv.getLocale()]: true };

    private langChange$ = this.localeSrv.onLangChange.pipe(takeUntilDestroyed())
        .subscribe((event: LangChangeEvent) => {
            const title = `PgBanned().onLangChange(${event.lang})`;
            if (!this.loadedLangs[event.lang] && !!this.translateSrv.translations) {
                if (environment.logLevel > 0) {
                    console.info(`${title} translate.setTranslation(${event.lang}); trans:`, { ...event.translations });
                }
                this.loadedLangs[event.lang] = true;
                // Add translations from the main module to this module.
                this.translateSrv.setTranslation(event.lang, event.translations, true);
            }
            this.translateSrv.use(event.lang);
        });

    public blockedUsers: BlockedUserDto[] = this.route.snapshot.data["blockedUsers"] || [];
    public isLoading: boolean = false;
    public sortColumn: string = SORT_COL_INIT;
    public sortDesc: boolean = SORT_DESC_INIT;

    constructor() {
        if (environment.logLevel > 0) { console.info(`PgBanned(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
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
                this.alertSrv.showError(errMsg, "pg-banned.error_get_blocked_users");
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
        this.isLoading = true;
        this.chatMessageSrv.deleteBlockedUser(nickname)
            .then(() => {
                Promise.resolve().then(() =>
                    this.doSort({ [this.sortColumn]: this.sortDesc }));
            })
            .catch((err: HttpErrorResponse) => {
                const errMsg = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(errMsg, "pg-banned.error_get_blocked_users");
                throw err;
            })
            .finally(() => {
                this.isLoading = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **

}


/*
import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from "@angular/core";
import { ActivatedRoute } from "@angular/router";
import { LocaleService } from "../common/locale.service";
import { BannedUsersComponent } from "../lib-banned/banned-users/banned-users.component";
import { BlockedUserDto } from "../lib-chat/chat-message-api.interface";
import { chatMessageSrv } from "../lib-chat/chat-message.service";
import { AlertService } from "../lib-dialog/alert.service";
import { HttpErrorUtil } from "../utils/http-error.util";

@Component({
  selector: "app-pg-banned",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./pg-banned.html",
  styleUrl: "./pg-banned.scss",
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgBanned {
    private route: ActivatedRoute = inject(ActivatedRoute);
    public blockedUsers: BlockedUserDto[] = this.route.snapshot.data["blockedUsers"] || [];
    public isLoading: boolean = false;
    public sortColumn: string = SORT_COL_INIT;
    public sortDesc: boolean = SORT_DESC_INIT;

    public localeService: LocaleService = inject(LocaleService);

    private alertService: AlertService = inject(AlertService);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private chatMessageSrv: chatMessageSrv = inject(chatMessageSrv);

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
            .catch((errHttp: HttpErrorResponse) => {
                console.error(`GetBlockedUsersError:`, errHttp);
                const errMsg = HttpErrorUtil.getMsgs(errHttp)[0];
                this.alertService.showError(errMsg, `pg-banned.error_get_blocked_users`);

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
        this.isLoading = true;
        this.chatMessageSrv.deleteBlockedUser(nickname)
            .then(() => {
                Promise.resolve().then(() =>
                    this.doSort({ [this.sortColumn]: this.sortDesc }));
            })
            .catch((errHttp: HttpErrorResponse) => {
                console.error(`GetBlockedUsersError:`, errHttp);
                const errMsg = HttpErrorUtil.getMsgs(errHttp)[0];
                this.alertService.showError(errMsg, `pg-banned.error_get_blocked_users`);
            })
            .finally(() => {
                this.isLoading = false;
                this.changeDetector.markForCheck();
            });
    }
}
*/