import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgProfileDetails } from './pg-profile-details';

describe('PgProfileDetails', () => {
  let component: PgProfileDetails;
  let fixture: ComponentFixture<PgProfileDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgProfileDetails]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgProfileDetails);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
