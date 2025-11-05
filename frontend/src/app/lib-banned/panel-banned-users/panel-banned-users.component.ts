import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { TranslatePipe } from '@ngx-translate/core';
import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';

@Component({
    selector: 'app-panel-banned-users',
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, TranslatePipe],
    templateUrl: './panel-banned-users.component.html',
    styleUrl: './panel-banned-users.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelBannedUsersComponent {
    @Input()
    public blockedUsers: BlockedUserDto[] = [];
    @Input()
    public locale: string | null | undefined;

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };

    constructor() {
    }

}
