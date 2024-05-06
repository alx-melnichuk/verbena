import { ChangeDetectionStrategy, ChangeDetectorRef, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { PanelStreamEditorComponent } from '../panel-stream-editor/panel-stream-editor.component';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';

import { ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

import { StreamService } from '../stream.service';
import { StreamDto, UpdateStreamFileDto } from '../stream-api.interface';

@Component({
  selector: 'app-stream-edit',
  standalone: true,
  imports: [CommonModule, SpinnerComponent, PanelStreamEditorComponent],
  templateUrl: './stream-edit.component.html',
  styleUrls: ['./stream-edit.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamEditComponent {
  public isLoadDataStream = false;
  public streamDto: StreamDto;
  
  private goBackToRoute: string = ROUTE_STREAM_LIST;

  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private streamService: StreamService,
    private alertService: AlertService,
  ) {
    console.log(`StreamEditComponent()`); // #-
    this.streamDto = this.route.snapshot.data['streamDto'];
    console.log(`StreamEditComponent() streamDto: `, this.streamDto); // #-
    const previousNavigation = this.router.getCurrentNavigation()?.previousNavigation;
    if (!!previousNavigation && !!previousNavigation.finalUrl) {
        this.goBackToRoute = previousNavigation.finalUrl.toString();
    }
  }

  // ** Public API **

  public doCancelStream(): void {
    this.goBack();
  }

  public doUpdateStream(updateStreamFile: UpdateStreamFileDto): void {
    this.alertService.hide();
    if (!updateStreamFile || (!updateStreamFile.createStreamDto && !updateStreamFile.modifyStreamDto)) {
      return;
    }
    const isEdit = (!!updateStreamFile.modifyStreamDto);
    const buffPromise: Promise<unknown>[] = [];
    this.isLoadDataStream = true;
    
    if (!!updateStreamFile.createStreamDto) {
      buffPromise.push(
        this.streamService.createStream(updateStreamFile.createStreamDto, updateStreamFile.logoFile || undefined)
      );
    } else if (!!updateStreamFile.id && !!updateStreamFile.modifyStreamDto) {
      buffPromise.push(
        this.streamService.modifyStream(updateStreamFile.id, updateStreamFile.modifyStreamDto, updateStreamFile.logoFile)
      );
    }
    Promise.all(buffPromise)
      .then((responses) => {
        Promise.resolve()
          .then(() => {
            this.goBack();
          });
      })
      .catch((error: HttpErrorResponse) => {
        console.error(`error: `, error); // #
        const title = (isEdit ? 'stream_edit.error_editing_stream' : 'stream_edit.error_creating_stream');
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title);
        throw error;
      })
      .finally(() => {
        this.isLoadDataStream = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Private API **

  private goBack(): Promise<boolean> {
    return this.router.navigateByUrl(this.goBackToRoute);
  }
}
