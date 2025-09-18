import {
    AfterContentInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, inject, Input,
    Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { LocaleService } from 'src/app/common/locale.service';
import { StringDateTime } from 'src/app/common/string-date-time';
import { AvatarComponent } from 'src/app/components/avatar/avatar.component';
import { SidebarHandlerDirective } from 'src/app/components/sidebar/sidebar-handler.directive';
import { SidebarComponent } from 'src/app/components/sidebar/sidebar.component';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { ChatMessageDto, ParamQueryPastMsg } from 'src/app/lib-chat/chat-message-api.interface';
import { PanelChatComponent } from 'src/app/lib-chat/panel-chat/panel-chat.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamDto, StreamState } from 'src/app/lib-stream/stream-api.interface';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';

import { PanelStreamActionsComponent } from '../panel-stream-actions/panel-stream-actions.component';
import { PanelStreamParamsComponent } from '../panel-stream-params/panel-stream-params.component';
import { PanelStreamStateComponent } from '../panel-stream-state/panel-stream-state.component';

@Component({
    selector: 'app-concept-view',
    exportAs: 'appConceptView',
    standalone: true,
    imports: [CommonModule, AvatarComponent, SpinnerComponent, SidebarComponent, TranslatePipe, PanelStreamStateComponent,
        PanelStreamParamsComponent, PanelStreamActionsComponent, PanelChatComponent, SidebarHandlerDirective],
    templateUrl: './concept-view.component.html',
    styleUrl: './concept-view.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class ConceptViewComponent implements AfterContentInit {
    @Input()
    public avatar: string | null | undefined;

    @Input() // List of new blocked users.
    public chatBlockedUsers: string[] = [];
    @Input() // List of past chat messages.
    public chatPastMsgs: ChatMessageDto[] = [];
    @Input() // List of new chat messages.
    public chatNewMsgs: ChatMessageDto[] = [];
    @Input() // List of IDs of permanently deleted chat messages.
    public chatRmvIds: number[] = [];
    @Input() // Indication that the user is blocked.
    public chatIsBlocked: boolean | null = null;
    @Input() // Indicates that the user can send messages to the chat.
    public chatIsEditable: boolean | null = null;
    @Input() // Indicates that data is being loaded.
    public chatIsLoadData: boolean | null = null;
    @Input() // Indicates that the user is the owner of the chat.
    public chatIsOwner: boolean | null = null;
    @Input()
    public chatMaxLen: number | null = null;
    @Input()
    public chatMinLen: number | null = null;
    @Input()
    public chatMaxRows: number | null = null;
    @Input()
    public chatMinRows: number | null = null;
    @Input()
    public chatNickname: string | null = null;

    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public isLoadStream = false;
    @Input()
    public isShowTimer: boolean = false;
    @Input()
    public isStreamOwner: boolean = false;
    @Input()
    public nickname: string | null = null;

    @Input()
    public streamDto: StreamDto | null = null;

    @Output()
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();
    @Output()
    readonly blockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly sendMessage: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMessage: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly cutMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly rmvMessage: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly queryPastMsgs: EventEmitter<ParamQueryPastMsg> = new EventEmitter();

    public isSidebarLfOpen: boolean = false;
    public isSidebarRgOpen: boolean = true; // false;
    public localeService: LocaleService = inject(LocaleService);

    // To disable the jumping effect of the "stream-video" panel at startup.
    public isStreamVideo = false;

    // Block "Chat"

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogService: DialogService = inject(DialogService);
    private translateService: TranslateService = inject(TranslateService);

    constructor() {
    }

    // To disable the jumping effect of the "stream-video" panel at startup.
    ngAfterContentInit(): void {
        this.isStreamVideo = true;
        this.changeDetector.markForCheck();
    }

    // ** Public API **

    // ** Side Left and Right **

    public clearEvent(event: Event): void {
        event.preventDefault();
        event.stopPropagation();
    }

    // Section: "panel stream admin"

    public getDate(starttime: StringDateTime | null | undefined): Date | null {
        return StringDateTimeUtil.toDate(starttime);
    }

    public doChangeState(newState: StreamState | undefined): void {
        this.changeState.emit(newState);
    }

    // Section: "Chat"

    public doBlockUser(user_name: string): void {
        if (!!user_name) {
            this.blockUser.emit(user_name);
        }
    }
    public doUnblockUser(user_name: string): void {
        if (!!user_name) {
            this.unblockUser.emit(user_name);
        }
    }
    public doSendMessage(newMessage: string): void {
        if (!!newMessage) {
            this.sendMessage.emit(newMessage);
        }
    }
    public doEditMessage(keyValue: KeyValue<number, string>): void {
        if (!!keyValue && !!keyValue.key) {
            this.editMessage.emit(keyValue);
        }
    }
    public async doCutMessage(keyValue: KeyValue<number, string>): Promise<void> {
        if (!keyValue || !keyValue.key || !keyValue.value) {
            return Promise.resolve();
        }
        const msg = keyValue.value.slice(0, 45) + (keyValue.value.length > 45 ? '...' : '');
        const message = this.translateService.instant('concept-view.sure_you_want_delete_message', { message: msg });
        const res = await this.dialogService.openConfirmation(
            message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' });
        if (!!res) {
            this.cutMessage.emit(keyValue.key);
        }
    }
    public async doRmvMessage(chMsgId: number): Promise<void> {
        if (!chMsgId || chMsgId < 0) {
            return Promise.resolve();
        }
        const message = this.translateService.instant('concept-view.sure_you_want_permanently_delete_message');
        const res = await this.dialogService.openConfirmation(
            message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' });
        if (!!res) {
            this.rmvMessage.emit(chMsgId);
        }
    }
    public doQueryPastMsgs(info: ParamQueryPastMsg) {
        this.queryPastMsgs.emit(info);
    }

    // ** Private API **

    // Updating data by stream

    private wait(ms: number): void {
        const start = Date.now();
        let now = start;
        while (now - start < ms) {
            now = Date.now();
        }
    }

}
