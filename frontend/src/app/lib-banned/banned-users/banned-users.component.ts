import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, inject, Input, Output, ViewEncapsulation } from '@angular/core';
import { PanelBannedUsersComponent } from '../panel-banned-users/panel-banned-users.component';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';
import { LocaleService } from 'src/app/common/locale.service';
import { TranslateService } from '@ngx-translate/core';
import { DialogService } from 'src/app/lib-dialog/dialog.service';

@Component({
    selector: 'app-banned-users',
    standalone: true,
    imports: [CommonModule, PanelBannedUsersComponent],
    templateUrl: './banned-users.component.html',
    styleUrl: './banned-users.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class BannedUsersComponent {
    @Input()
    public blockedUsers: BlockedUserDto[] = [];
    @Input()
    public isLoading: boolean | null = null;
    @Input()
    public sortColumn: string | undefined | null;
    @Input()
    public sortDesc: boolean | undefined | null;

    @Output()
    readonly sort: EventEmitter<Record<string, boolean>> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();

    public title: string | null = 'panel-banned-users.title';

    public localeService: LocaleService = inject(LocaleService);

    private dialogService: DialogService = inject(DialogService);
    private translateService: TranslateService = inject(TranslateService);

    constructor() {
    }

    // ** Public API **

    public doSort(event: Record<string, boolean>): void {
        this.sort.emit(event);
    }

    public async doUnblockUser(nickname: string): Promise<void> {
        if (!nickname) {
            return;
        }
        const message = this.translateService.instant('banned-users.are_you_want_to_unblock_user', { nickname });
        const res = await this.dialogService.openConfirmation(
            message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' });
        if (!!res) {
            this.unblockUser.emit(nickname);
        }
    }
}