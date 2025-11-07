import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { LocaleService } from '../common/locale.service';
import { PanelBannedUsersComponent } from '../lib-banned/panel-banned-users/panel-banned-users.component';
import { BlockedUserDto } from '../lib-chat/chat-message-api.interface';

@Component({
    selector: 'app-pg-banned',
    standalone: true,
    imports: [CommonModule, PanelBannedUsersComponent],
    templateUrl: './pg-banned.component.html',
    styleUrl: './pg-banned.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgBannedComponent {
    public blockedUsers: BlockedUserDto[] = [];

    public localeService: LocaleService = inject(LocaleService);

    private route: ActivatedRoute = inject(ActivatedRoute);

    constructor() {
        this.blockedUsers = this.route.snapshot.data['blockedUsers'];
    }
}
