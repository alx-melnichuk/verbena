import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { LocaleService } from 'src/app/common/locale.service';
import { BlockedUserDto } from 'src/app/lib-chat/chat-message-api.interface';
import { PanelBannedUsersComponent } from '../panel-banned-users/panel-banned-users.component';

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
    public blockedUsers: BlockedUserDto[] = [];

    public localeService: LocaleService = inject(LocaleService);

    private route: ActivatedRoute = inject(ActivatedRoute);

    constructor() {
        this.blockedUsers = this.route.snapshot.data['blockedUsers'];
    }

}
