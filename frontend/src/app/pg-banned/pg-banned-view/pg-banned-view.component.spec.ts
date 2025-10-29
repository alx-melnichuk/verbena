import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBannedViewComponent } from './pg-banned-view.component';

describe('PgBannedViewComponent', () => {
  let component: PgBannedViewComponent;
  let fixture: ComponentFixture<PgBannedViewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBannedViewComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBannedViewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
