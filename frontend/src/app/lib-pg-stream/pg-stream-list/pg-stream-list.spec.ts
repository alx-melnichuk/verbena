import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgStreamList } from './pg-stream-list';

describe('PgStreamList', () => {
  let component: PgStreamList;
  let fixture: ComponentFixture<PgStreamList>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgStreamList]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgStreamList);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
