import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgAboutDetails } from './pg-about-details';

describe('PgAboutDetails', () => {
  let component: PgAboutDetails;
  let fixture: ComponentFixture<PgAboutDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgAboutDetails]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgAboutDetails);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
