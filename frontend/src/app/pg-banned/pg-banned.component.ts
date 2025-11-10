import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { LocaleService } from '../common/locale.service';
import { BannedUsersComponent } from '../lib-banned/banned-users/banned-users.component';
import { BlockedUserDto } from '../lib-chat/chat-message-api.interface';
import { ChatMessageService } from '../lib-chat/chat-message.service';
import { AlertService } from '../lib-dialog/alert.service';
import { HttpErrorUtil } from '../utils/http-error.util';

export const SORT_COL_INIT: string = 'nickname';
export const SORT_DESC_INIT: boolean = false;

@Component({
    selector: 'app-pg-banned',
    standalone: true,
    imports: [CommonModule, BannedUsersComponent],
    templateUrl: './pg-banned.component.html',
    styleUrl: './pg-banned.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgBannedComponent {
    public blockedUsers: BlockedUserDto[] = [];
    public isLoading: boolean = false;
    public sortColumn: string = SORT_COL_INIT;
    public sortDesc: boolean = SORT_DESC_INIT;

    public localeService: LocaleService = inject(LocaleService);

    private alertService: AlertService = inject(AlertService);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private chatMessageService: ChatMessageService = inject(ChatMessageService);

    constructor() {
        this.blockedUsers = this.route.snapshot.data['blockedUsers'];
    }

    // ** Public API **

    public doSort(event: Record<string, boolean>): void {
        const keys = Object.keys(event);
        const sortColumn = keys.length > 0 ? keys[0] : '';
        const sortDesc: boolean | undefined = !!sortColumn ? event[sortColumn] : false;
        if (!sortColumn) {
            return;
        }
        this.isLoading = true;
        this.chatMessageService.getBlockedUsers(sortColumn, sortDesc)
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
        this.chatMessageService.deleteBlockedUser(nickname)
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
