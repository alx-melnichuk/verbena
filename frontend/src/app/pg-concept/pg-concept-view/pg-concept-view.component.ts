import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, Input, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { StreamDto } from 'src/app/lib-stream/stream-api.interface';

@Component({
    selector: 'app-pg-concept-view',
    standalone: true,
    imports: [CommonModule, ConceptViewComponent],
    templateUrl: './pg-concept-view.component.html',
    styleUrl: './pg-concept-view.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgConceptViewComponent implements OnInit {

    @Input()
    public streamDto: StreamDto | null = null;

    constructor(
        private route: ActivatedRoute,
    ) {
        this.streamDto = this.route.snapshot.data['streamDto'];
    }

    ngOnInit(): void {
        // console.log(`^OnInit()_this.streamDto:`, this.streamDto); // #
    }

}
