import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, Input, Output, ViewEncapsulation } from '@angular/core';
import { MatMenuModule } from '@angular/material/menu';
import { TranslatePipe } from '@ngx-translate/core';
import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';

@Component({
    selector: 'app-panel-blocked-users',
    standalone: true,
    imports: [CommonModule, MatMenuModule, DateTimeFormatPipe, SpinnerComponent, TranslatePipe],
    templateUrl: './panel-blocked-users.component.html',
    styleUrl: './panel-blocked-users.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelBlockedUsersComponent {
    @Input()
    public blockedUsers: BlockedUserDto[] = []; // List of blocked users.
    @Input() // Indicates that data is being loaded.
    public isLoadData: boolean | null = null;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public title: string | null = null;

    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };


    constructor() {
    }

    // ** Public API **

    public doUnblockUser(member: string | null | undefined, blockedUsers: BlockedUserDto[]): void {
        if (!!member && !!blockedUsers && blockedUsers.findIndex((v) => member == v.blockedNickname) > -1) {
            this.unblockUser.emit(member);
        }
    }

}
