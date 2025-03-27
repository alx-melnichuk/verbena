import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

import { StreamListComponent } from 'src/app/lib-stream/stream-list/stream-list.component';
import { StreamService } from 'src/app/lib-stream/stream.service';

@Component({
    selector: 'app-pg-stream-list',
    standalone: true,
    imports: [CommonModule, StreamListComponent],
    templateUrl: './pg-stream-list.component.html',
    styleUrl: './pg-stream-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamListComponent {

    public profileDto: ProfileDto | null;

    constructor(
        private route: ActivatedRoute,
        private streamService: StreamService,
    ) {
        this.profileDto = this.route.snapshot.data['profileDto'];
    }

    // ** Public API **

    public doActionDuplicate(streamId: number): void {
        this.streamService.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.streamService.redirectToStreamEditingPage(streamId);
    }

    public async doActionDelete(streamId: number): Promise<void> {
        if (!streamId) {
            return Promise.resolve();
        }
        await this.deleteDataStream(streamId);
    }

    // ** Private API **

    private async deleteDataStream(streamId: number): Promise<void> {

        if (!streamId) {
            return Promise.reject();
        }
        let isRefres = false;
        try {
            await this.streamService.deleteStream(streamId);
            isRefres = true;
        } catch (error) {
            //this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_delete_stream');
            //throw error;
        } finally {
            //this.changeDetector.markForCheck();
            // if (isRefres) {
            //     setTimeout(() => this.loadFutureAndPastStreamsAndSchedule(), 0);
            // }
        }
    }
}
