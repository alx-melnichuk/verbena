import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { TranslatePipe } from '@ngx-translate/core';
import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';

@Component({
    selector: 'app-panel-blocked-users',
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, SpinnerComponent, TranslatePipe],
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

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'medium', timeStyle: 'short' };
    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };

}
