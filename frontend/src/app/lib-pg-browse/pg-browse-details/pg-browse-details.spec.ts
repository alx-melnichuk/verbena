import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgBrowseDetails } from './pg-browse-details';

describe('PgBrowseDetails', () => {
  let component: PgBrowseDetails;
  let fixture: ComponentFixture<PgBrowseDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgBrowseDetails]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgBrowseDetails);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
