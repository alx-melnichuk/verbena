import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { TranslatePipe } from '@ngx-translate/core';
import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { AvatarComponent } from 'src/app/components/avatar/avatar.component';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';

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
    public locale: string | null | undefined;
    @Input()
    public title: string | null = 'panel-banned-users.title';

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };

    constructor() {
    }

}
