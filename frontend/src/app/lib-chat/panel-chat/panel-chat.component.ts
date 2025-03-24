import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

import { TranslatePipe } from '@ngx-translate/core';

interface ChatMsg {
    id: string;
    userId: string;
    nickname: string;
    avatar?: string;
    event: string;
    text: string;
}

@Component({
    selector: 'app-panel-chat',
    standalone: true,
    imports: [CommonModule, TranslatePipe],
    templateUrl: './panel-chat.component.html',
    styleUrl: './panel-chat.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelChatComponent {
    @Input()
    public label = '';

    public chatMsgs = this.getChatMsg();


    private getChatMsg(): ChatMsg[] {
        const result: ChatMsg[] = [];


        return result;
    }
}
