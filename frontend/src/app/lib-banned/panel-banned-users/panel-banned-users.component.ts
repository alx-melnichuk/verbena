import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, inject, Input, Output, ViewEncapsulation } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { TranslatePipe } from '@ngx-translate/core';
import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { AvatarComponent } from 'src/app/components/avatar/avatar.component';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';

const COL_NICKNAME = 'nickname';
const COL_EMAIL = 'email';
const COL_BLOCK_DATE = 'block_date';

@Component({
    selector: 'app-panel-banned-users',
    standalone: true,
    imports: [CommonModule, MatButtonModule, AvatarComponent, DateTimeFormatPipe, TranslatePipe],
    templateUrl: './panel-banned-users.component.html',
    styleUrl: './panel-banned-users.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelBannedUsersComponent {
    @Input()
    public blockedUsers: BlockedUserDto[] = [];
    @Input()
    public isLoading: boolean | null = null;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public title: string | null = 'panel-banned-users.title';
    @Input()
    public sortColumn: string | undefined | null;
    @Input()
    public sortDesc: boolean | undefined | null;

    @Output()
    readonly sort: EventEmitter<Record<string, boolean>> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };
    readonly colNickname: string = COL_NICKNAME;
    readonly colEmail: string = COL_EMAIL;
    readonly colBlockDate: string = COL_BLOCK_DATE;

    // ** Public API **

    public doSort(newColumn: string, sortColumn: string | undefined | null, sortDesc: boolean): void {
        if (!!newColumn) {
            const newDesc = newColumn == sortColumn ? !sortDesc : false;
            this.sort.emit({ [newColumn]: newDesc });
        }
    }

    public doUnblockUser(nickname: string): void {
        if (!!nickname) {
            this.unblockUser.emit(nickname);
        }
    }
}
